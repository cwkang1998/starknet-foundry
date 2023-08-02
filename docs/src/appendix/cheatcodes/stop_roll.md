# `stop_roll`

> `fn stop_roll(contract_address: ContractAddress)`

Cancels the [`start_roll`](./start_roll.md) for the contract at the given address.

- `contract_address` - target contract address

```rust
#[test]
fn test_roll() {
    // ...
    
    stop_roll(contract_address);
    
    // ...
}
```