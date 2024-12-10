use bitcoincore_rpc::{Auth, Client, RpcApi};
use serde_json::json;
use async_trait::async_trait;

pub mod client {
    use super::*;

    #[async_trait]
    pub trait RpcClient {
        fn call(&self, method: &str, params: &[serde_json::Value]) -> Result<serde_json::Value, Box<dyn std::error::Error>>;
        fn get_balance(&self, account: Option<&str>, include_watchonly: Option<bool>) -> Result<f64, Box<dyn std::error::Error>>;
        async fn http_get(&self, url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    }

    pub struct RealRpcClient {
        client: Client,
    }

    impl RealRpcClient {
        pub fn new(rpc_url: &str, rpc_user: &str, rpc_password: &str) -> Result<Self, Box<dyn std::error::Error>> {
            let auth = Auth::UserPass(rpc_user.to_string(), rpc_password.to_string());
            let client = Client::new(rpc_url, auth)?;
            Ok(RealRpcClient { client })
        }

        pub fn create_or_load_wallet(&self, wallet_name: &str) -> Result<(), Box<dyn std::error::Error>> {
            let params = vec![json!(wallet_name)];
            let response: Result<(), _> = self.client.call::<()>("createwallet", &params);
            match response {
                Ok(_) => Ok(()),
                Err(e) if e.to_string().contains("already exists") => {
                    let load_params = vec![json!(wallet_name)];
                    self.client.call::<()>("loadwallet", &load_params).map_err(Box::new)?;
                    Ok(())
                },
                Err(e) => Err(Box::new(e)),
            }
        }
    }

    #[async_trait]
    impl RpcClient for RealRpcClient {
        fn call(&self, method: &str, params: &[serde_json::Value]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
            Ok(self.client.call(method, params)?)
        }

        fn get_balance(&self, account: Option<&str>, include_watchonly: Option<bool>) -> Result<f64, Box<dyn std::error::Error>> {
            // Convert Option<&str> to Option<usize>
            let account_index = account.map(|_| 0);
            let balance = self.client.get_balance(account_index, include_watchonly)?;
            Ok(balance.as_btc())
        }

        async fn http_get(&self, url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            let response = reqwest::get(url).await?.bytes().await?;
            Ok(response.to_vec())
        }
    }

    pub struct SyscoinClient<T: RpcClient> {
        rpc_client: T,
        poda_url: String,
    }

    impl<T: RpcClient> SyscoinClient<T> {
        pub fn new(rpc_client: T, poda_url: &str) -> Self {
            SyscoinClient {
                rpc_client,
                poda_url: poda_url.to_string(),
            }
        }

        pub async fn create_blob(&self, data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
            let data_str = hex::encode(data);
            let params = vec![json!(data_str)];
            let response = self.rpc_client.call("syscoincreatenevmblob", &params)?;
            Ok(response["result"]["versionhash"].as_str().unwrap().to_string())
        }

        pub fn get_balance(&self) -> Result<f64, Box<dyn std::error::Error>> {
            self.rpc_client.get_balance(None, None)
        }

        pub async fn get_blob_from_cloud(&self, version_hash: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            let url = format!("{}{}", self.poda_url, version_hash);
            self.rpc_client.http_get(&url).await
        }

        pub fn create_or_load_wallet(&self, wallet_name: &str) -> Result<(), Box<dyn std::error::Error>> {
            self.rpc_client.call("createwallet", &[json!(wallet_name)])?;
            Ok(())
        }
    }

    pub struct MockRpcClient;

    #[async_trait]
    impl RpcClient for MockRpcClient {
        fn call(&self, method: &str, _params: &[serde_json::Value]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
            match method {
                "syscoincreatenevmblob" => Ok(json!({"result": {"versionhash": "mocked_hash"}})),
                _ => Err(format!("Unsupported RPC method: {}", method).into()),
            }
        }

        fn get_balance(&self, _account: Option<&str>, _include_watchonly: Option<bool>) -> Result<f64, Box<dyn std::error::Error>> {
            Ok(100.0) // Mock balance
        }

        async fn http_get(&self, url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            if url.ends_with("mocked_version_hash") {
                Ok(vec![1, 2, 3]) // Mock blob data
            } else {
                Err("Mocked HTTP error".into())
            }
        }
    }
}
