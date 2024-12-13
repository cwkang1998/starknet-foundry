use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use camino::Utf8PathBuf;
use reqwest::StatusCode;
use serde::Serialize;
use sncast::response::structs::VerifyResponse;
use sncast::Network;
use starknet::core::types::FieldElement;
use std::ffi::OsStr;
use walkdir::WalkDir;

#[async_trait]
pub trait VerificationInterface {
    fn new(network: Network, workspace_dir: Utf8PathBuf) -> Self;
    async fn verify(
        &self,
        contract_address: Option<FieldElement>,
        class_hash: Option<FieldElement>,
        class_name: String,
    ) -> Result<VerifyResponse>;
    fn gen_explorer_url(&self) -> Result<String>;
}

pub struct BaseVerificationInterface {
    pub network: Network,
    pub workspace_dir: Utf8PathBuf,
}

impl BaseVerificationInterface {
    pub fn read_workspace_files(&self) -> Result<serde_json::Map<String, serde_json::Value>> {
        // Read all files name along with their contents in a JSON format
        // in the workspace dir recursively
        // key is the file name and value is the file content
        let mut file_data = serde_json::Map::new();

        // Recursively read files and their contents in workspace directory
        for entry in WalkDir::new(self.workspace_dir.clone()).follow_links(true) {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == OsStr::new("cairo") || extension == OsStr::new("toml") {
                        let relative_path = path.strip_prefix(self.workspace_dir.clone())?;
                        let file_content = std::fs::read_to_string(path)?;
                        file_data.insert(
                            relative_path.to_string_lossy().into_owned(),
                            serde_json::Value::String(file_content),
                        );
                    }
                }
            }
        }
        Ok(file_data)
    }

    pub async fn send_verification_request(
        &self,
        url: String,
        payload: VerificationPayload,
    ) -> Result<VerifyResponse> {
        let json_payload = serde_json::to_string(&payload)?;
        let client = reqwest::Client::new();
        let api_res = client
            .post(url)
            .header("Content-Type", "application/json")
            .body(json_payload)
            .send()
            .await
            .context("Failed to send request to verifier API")?;

        if api_res.status() == StatusCode::OK {
            let message = api_res
                .text()
                .await
                .context("Failed to read verifier API response")?;
            Ok(VerifyResponse { message })
        } else {
            let message = api_res.text().await.context("Failed to verify contract")?;
            Err(anyhow!(message))
        }
    }
}

#[derive(Serialize, Debug)]
pub struct VerificationPayload {
    pub class_name: String,
    pub contract_address: Option<FieldElement>,
    pub class_hash: Option<FieldElement>,
    pub source_code: serde_json::Value,
}
