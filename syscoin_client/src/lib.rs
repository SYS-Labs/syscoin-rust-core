pub mod client {
    use bitcoincore_rpc::bitcoin::Amount;
    use bitcoincore_rpc::{Auth, Client, RpcApi};
    use serde::{Deserialize, Serialize};
    use serde_json::{json, Value};

    /// Struct representing a client for interacting with Syscoin RPC and PoDA services.
    pub struct SyscoinClient {
        client: Client,
        poda_url: String,
    }

    impl SyscoinClient {
        /// Constructs a new `SyscoinClient` with the given RPC details and PoDA URL.
        pub fn new(rpc_url: &str, user: &str, password: &str, poda_url: &str) -> Self {
            let auth = Auth::UserPass(user.to_string(), password.to_string());
            let client = Client::new(rpc_url, auth).unwrap();
            SyscoinClient { client, poda_url: poda_url.to_string() }
        }

        /// Creates a blob on the Syscoin blockchain with the provided data.
        pub fn create_blob(&self, data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
            let data_str = hex::encode(data);
            let params = vec![json!(data_str)];
            let response: serde_json::Value = self.client.call("syscoincreatenevmblob", &params)?;
            Ok(response["result"]["versionhash"].as_str().unwrap().to_string())
        }

        /// Retrieves the balance from the Syscoin wallet.
        pub fn get_balance(&self) -> Result<f64, Box<dyn std::error::Error>> {
            let balance: Amount = self.client.get_balance(None, None)?;
            Ok(balance.as_btc())
        }

        /// Fetches blob data from a cloud service using the version hash.
        pub async fn get_blob_from_cloud(&self, version_hash: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            let url = format!("{}{}", self.poda_url, version_hash);
            let response = reqwest::get(url).await?.bytes().await?;
            let data = hex::decode(response)?;
            Ok(data)
        }

        /// Generates a new address with an optional label.
        pub async fn get_new_address(&self, label: &str) -> Result<String, Box<dyn std::error::Error>> {
            let address = self.client.get_new_address(Some(label), None)?;
            Ok(address.to_string())
        }

        /// Attempts to fetch an address associated with a given label.
        pub async fn fetch_address_by_label(&self, label: &str) -> Result<String, Box<dyn std::error::Error>> {
            let params = vec![json!(label)];
            let response: serde_json::Value = self.client.call("getaddressesbylabel", &params)?;
            if let Some(result) = response["result"].as_object() {
                if let Some((address, _)) = result.iter().next() {
                    return Ok(address.clone());
                }
            }
            Err("No address found for the label".into())
        }

        /// Retrieves the current block number.
        pub async fn block_number(&self) -> Result<u64, Box<dyn std::error::Error>> {
            let block_count = self.client.get_block_count()?;
            Ok(block_count as u64)
        }

        /// Attempts to get a transaction receipt for a blob using its version hash.
        pub async fn transaction_receipt(&self, version_hash: &str) -> Result<Option<Value>, Box<dyn std::error::Error>> {
            let params = vec![json!(version_hash)];
            let response: Value = self.client.call("getnevmblobdata", &params)?;
            let result = response["result"].as_object().map(|map| Value::Object(map.clone()));
            Ok(result)
        }

        /// Creates or loads a wallet with the given name.
        pub fn create_or_load_wallet(&self, wallet_name: &str) -> Result<(), Box<dyn std::error::Error>> {
            let params = vec![json!(wallet_name)];
            let response: serde_json::Value = self.client.call("createwallet", &params)?;
            // Check if the wallet was created or if it already exists
            if response["error"].as_object().is_none() {
                println!("Wallet {} created or loaded.", wallet_name);
            } else {
                let error_msg = response["error"]["message"].as_str().unwrap_or("Unknown error");
                // Try to load the wallet if creation failed (assuming it exists)
                if error_msg.contains("already exists") {
                    let params = vec![json!(wallet_name)];
                    let load_response: serde_json::Value = self.client.call("loadwallet", &params)?;
                    if load_response["error"].as_object().is_none() {
                        println!("Wallet {} loaded.", wallet_name);
                    } else {
                        let load_error_msg = load_response["error"]["message"].as_str().unwrap_or("Unknown error");
                        return Err(format!("Failed to load wallet: {}", load_error_msg).into());
                    }
                } else {
                    return Err(format!("Failed to create or load wallet: {}", error_msg).into());
                }
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    mod get_balance;
    mod create_blob;
    // Add other test modules here as you create them
}