use crate::client::SyscoinClient;

#[tokio::test]
async fn test_create_blob() {
    let client = SyscoinClient::new("http://localhost:8370", "u", "p", "podaurl/");
    let data = vec![1, 2, 3];
    let blob_hash = client.create_blob(&data).await.unwrap();
    assert!(!blob_hash.is_empty(), "Blob hash should not be empty");
}