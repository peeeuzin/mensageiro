use async_trait::async_trait;
use log::debug;
use serde::Serialize;
use std::{collections::HashMap, future::Future, net::SocketAddr, sync::Arc};
use tokio::{
    io::{self, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{TcpListener, TcpStream, ToSocketAddrs},
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        RwLock,
    },
};

use uuid::Uuid;

mod spec;

pub use spec::*;

const BUFFER_SIZE: usize = 1024;

type Tx = UnboundedSender<Response>;
type Rx = UnboundedReceiver<Response>;
pub type SharedState = Arc<RwLock<Shared>>;

pub struct Shared {
    peers: HashMap<Uuid, Peer>,
}

impl Shared {
    pub fn new() -> Self {
        Self {
            peers: Default::default(),
        }
    }

    pub fn get_peer(&self, id: Uuid) -> Option<&Peer> {
        self.peers.get(&id)
    }
}

impl Default for Shared {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Peer {
    tx: Tx,
}

impl Peer {
    pub fn new(tx: Tx) -> Self {
        Self { tx }
    }

    pub async fn send(&self, response: Response) {
        self.tx.send(response).ok();
    }
}

#[async_trait]
pub trait Wave: Sync + Send + 'static {
    async fn request(&self, request: Request, uuid: Uuid) -> Response;
    async fn new_connection(&self, stream: &mut TcpStream, addr: SocketAddr) -> bool;
}

pub struct WaveService<T: Wave> {
    inner: Arc<T>,
    state: Arc<RwLock<Shared>>,
}

impl<T: Wave> WaveService<T> {
    pub fn new(inner: T, state: Arc<RwLock<Shared>>) -> Self {
        let inner = Arc::new(inner);

        Self { inner, state }
    }
}

impl<T: Wave> Clone for WaveService<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            state: self.state.clone(),
        }
    }
}

pub struct Server<T: Wave> {
    svc: WaveService<T>,
}

impl<T: Wave> Server<T> {
    pub fn new(svc: WaveService<T>) -> Self {
        Self { svc }
    }

    pub async fn serve<A: ToSocketAddrs>(self, addr: A) {
        let listener = TcpListener::bind(addr).await.unwrap();

        loop {
            let (mut stream, addr) = listener.accept().await.unwrap();

            let svc = self.svc.clone();
            let state = svc.state.clone();

            tokio::spawn(async move {
                if svc.inner.new_connection(&mut stream, addr).await {
                    let (tx, rx) = mpsc::unbounded_channel();
                    let id = Uuid::new_v4();

                    {
                        let mut state = state.write().await;
                        state.peers.insert(id, Peer::new(tx.clone()));
                    }

                    handle_connection(stream, (tx, rx), |request| svc.inner.request(request, id))
                        .await;
                } else {
                    terminate_connection(stream).await.ok();
                }
            });
        }
    }
}

fn process_request(buf: &[u8]) -> Result<Request, Response> {
    let request: Request = match bson::from_slice(buf) {
        Ok(request) => request,
        Err(_) => {
            let response = Response {
                body: None,
                path: None,
                headers: Default::default(),
                kind: Type::Generic,
                status: Status::Unparseable,
            };

            return Err(response);
        }
    };

    Ok(request)
}

pub async fn read_socket<IO>(socket: &mut IO, buffer: &mut [u8]) -> io::Result<Vec<u8>>
where
    IO: AsyncRead + Unpin,
{
    let mut request_bytes = Vec::new();

    while let Ok(n) = socket.read(buffer).await {
        if n == 0 {
            break;
        }

        request_bytes.extend_from_slice(&buffer[..n]);

        if n < BUFFER_SIZE {
            break;
        }
    }

    Ok(request_bytes)
}

pub async fn write_socket<IO, D>(socket: &mut IO, data: &D) -> io::Result<()>
where
    IO: AsyncWrite + Unpin,
    D: Sized + Serialize,
{
    let data = bson::to_vec(data).unwrap();

    socket.write_all(&data).await
}

async fn handle_connection<F, O, IO>(socket: IO, (tx, mut rx): (Tx, Rx), callback: F)
where
    F: Fn(Request) -> O,
    O: Future<Output = Response>,
    IO: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let (mut read, mut write) = tokio::io::split(socket);

    // Spawn a task to write responses to the socket
    tokio::spawn(async move {
        while let Some(response) = rx.recv().await {
            debug!("Sending response. Status: {:?}", response.status);
            write_socket(&mut write, &response).await.ok();
        }
    });

    // Buffer to read data from the socket
    let mut buffer = vec![0; BUFFER_SIZE];

    // Read data from the socket
    loop {
        let request_bytes = read_socket(&mut read, &mut buffer).await.unwrap();

        if request_bytes.is_empty() {
            break;
        }

        match process_request(&request_bytes[..]) {
            Ok(request) => {
                let response = callback(request).await;

                tx.send(response).ok();
            }
            Err(response) => {
                tx.send(response).ok();
            }
        }
    }
}

async fn terminate_connection<IO>(mut socket: IO) -> io::Result<()>
where
    IO: AsyncWrite + Unpin,
{
    socket.shutdown().await
}
