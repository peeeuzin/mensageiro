use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::*;

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub async fn connect(addr: &str) -> Self {
        let stream = TcpStream::connect(addr).await.unwrap();

        Self { stream }
    }

    pub async fn send(&mut self, request: Request) -> Response {
        let buf = bson::to_vec(&request).unwrap();
        self.stream.write_all(&buf).await.unwrap();

        let mut buf = vec![0; 1024];
        let n = self.stream.read(&mut buf).await.unwrap();

        bson::from_slice(&buf[..n]).unwrap()
    }
}
