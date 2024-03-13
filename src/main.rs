use std::sync::Arc;

use async_trait::async_trait;
use log::info;

mod core;
mod log_util;
mod server;

use rayon::{ThreadPool, ThreadPoolBuilder};
use server::*;
use tokio::{net::TcpStream, sync::RwLock};

struct MessageiroServer {
    core: core::Core,
    pool: ThreadPool,
}

#[async_trait]
impl Messageiro for MessageiroServer {
    async fn request(&self, request: Request, uuid: uuid::Uuid) -> Response {
        self.pool
            .install(move || futures::executor::block_on(self.core.handle_request(request, uuid)))
    }

    async fn new_connection(&self, _: &mut TcpStream, addr: std::net::SocketAddr) -> bool {
        info!("New connection from: {}", addr);

        true
    }
}

#[tokio::main]
async fn main() {
    init_logger();

    initialize_server().await;
}

pub async fn initialize_server() {
    let addr = "0.0.0.0:6572";

    let pool = ThreadPoolBuilder::new().num_threads(4).build().unwrap();
    let state = Arc::new(RwLock::new(Shared::new()));

    let messageiro_server = MessageiroServer {
        core: core::Core::new(state.clone()),
        pool,
    };

    let server = Server::new(MessageiroService::new(messageiro_server, state));

    info!("Wave is listening on port: {}", addr);

    server.serve(addr).await;
}

fn init_logger() {
    log::set_logger(&log_util::Logger)
        .map(|()| log::set_max_level(log::LevelFilter::Debug))
        .ok();
}
