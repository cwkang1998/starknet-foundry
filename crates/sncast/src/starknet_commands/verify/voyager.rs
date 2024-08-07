use super::base::{BaseVerificationInterface, VerificationInterface, VerificationPayload};
use anyhow::Result;
use async_trait::async_trait;
use camino::Utf8PathBuf;
use sncast::response::structs::VerifyResponse;
use sncast::Network;
use starknet::core::types::FieldElement;
use std::env;

pub struct VoyagerVerificationInterface {
    base: BaseVerificationInterface,
}

#[async_trait]
impl VerificationInterface for VoyagerVerificationInterface {
    fn new(network: Network, workspace_dir: Utf8PathBuf) -> Self {
        VoyagerVerificationInterface {
            base: BaseVerificationInterface {
                network,
                workspace_dir,
            },
        }
    }

    async fn verify(
        &self,
        contract_address: FieldElement,
        contract_name: String,
    ) -> Result<VerifyResponse> {
        let file_data = self.base.read_workspace_files()?;
        let source_code = serde_json::Value::Object(file_data);
        let payload = VerificationPayload {
            contract_name,
            contract_address: contract_address.to_string(),
            source_code,
        };
        let url = self.gen_explorer_url()?;
        self.base.send_verification_request(url, payload).await
    }

    fn gen_explorer_url(&self) -> Result<String> {
        let api_base_url = env::var("VOYAGER_API_URL")
            .unwrap_or_else(|_| "https://api.voyager.online/beta".to_string());
        let path = match self.base.network {
            Network::Mainnet => "/v1/sn_main/verify",
            Network::Sepolia => "/v1/sn_sepolia/verify",
        };
        Ok(format!("{api_base_url}{path}"))
    }
}
