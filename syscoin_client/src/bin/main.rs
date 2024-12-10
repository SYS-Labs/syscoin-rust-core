use syscoin_client::client::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a real RPC client
    let rpc_url = "http://127.0.0.1:8370";
    let rpc_user = "user";
    let rpc_password = "password";
    let poda_url = "http://poda.syscoin.org/";

    let rpc_client = RealRpcClient::new(rpc_url, rpc_user, rpc_password)?;
    rpc_client.create_or_load_wallet("wallet_name")?;

    let syscoin_client = SyscoinClient::new(rpc_client, poda_url);

    // Example usage
    let balance = syscoin_client.get_balance()?;
    println!("Balance: {}", balance);

    let blob_hash = syscoin_client.create_blob(&[1, 2, 3, 4]).await?;
    println!("Created Blob Hash: {}", blob_hash);

    let blob_data = syscoin_client.get_blob_from_cloud(&blob_hash).await?;
    println!("Blob Data: {:?}", blob_data);

    Ok(())
} 