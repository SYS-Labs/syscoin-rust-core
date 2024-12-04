use crate::client::SyscoinClient;

#[tokio::test]
async fn test_get_balance() {
    let client = SyscoinClient::new("http://localhost:8370", "u", "p", "podaurl/");
    let balance = client.get_balance().await.unwrap();
    assert!(balance >= 0.0, "Balance should be non-negative");
}