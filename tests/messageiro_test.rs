use messageiro::*;

#[tokio::test]
pub async fn connect_messageiro() {
    let addr = "0.0.0.0:6572";

    let _client = Client::connect(addr).await;
}

#[tokio::test]
pub async fn send_generic_message_messageiro() {
    let addr = "0.0.0.0:6572";

    let mut client = Client::connect(addr).await;

    let request = Request {
        path: "/".to_string(),
        kind: Type::Generic,
        headers: Default::default(),
        body: Some(b"Hello, world!".to_vec()),
    };

    let response = client.send(request).await;

    assert_eq!(response.status, Status::Ok);
    assert_eq!(response.path, Some("/".to_string()));
    assert_eq!(response.kind, Type::Generic);
}

#[tokio::test]
pub async fn send_publish_message_messageiro() {
    let addr = "0.0.0.0:6572";

    let mut client = Client::connect(addr).await;

    let request = Request {
        path: "/".to_string(),
        kind: Type::Publish,
        headers: Default::default(),
        body: Some(b"Hello, world!".to_vec()),
    };

    let response = client.send(request).await;

    assert_eq!(response.status, Status::Ok);
    assert_eq!(response.path, Some("/".to_string()));
    assert_eq!(response.kind, Type::Publish);
}

#[tokio::test]
pub async fn send_subscribe_message_messageiro() {
    let addr = "0.0.0.0:6572";

    let mut client = Client::connect(addr).await;

    let request = Request {
        path: "/".to_string(),
        kind: Type::Subscribe,
        headers: Default::default(),
        body: None,
    };

    let response = client.send(request).await;

    assert_eq!(response.status, Status::Ok);
    assert_eq!(response.path, Some("/".to_string()));
    assert_eq!(response.kind, Type::Subscribe);
}
