use std::{collections::HashMap, sync::Arc};

use crate::*;

use log::info;
use tokio::sync::RwLock;

pub struct Core {
    state: SharedState,
    inner: Arc<RwLock<HashMap<String, Vec<uuid::Uuid>>>>,
}

impl Core {
    pub fn new(state: SharedState) -> Self {
        info!("Initializing core");
        Self {
            state,
            inner: Default::default(),
        }
    }

    pub async fn handle_request(&self, request: Request, uuid: uuid::Uuid) -> Response {
        match request.kind {
            Type::Publish => self.publish(request, uuid).await,
            Type::Subscribe => self.subscribe(request, uuid).await,
            Type::Unsubscribe => self.unsubscribe(request, uuid).await,
            Type::Generic => self.generic(request),

            _ => Response {
                kind: Type::Generic,
                path: Some(request.path),
                status: Status::BadRequest,
                headers: Default::default(),
                body: None,
            },
        }
    }

    async fn publish(&self, request: Request, uuid: uuid::Uuid) -> Response {
        let state = self.state.write().await;
        let inner = self.inner.read().await;

        if let Some(peers) = inner.get(&request.path) {
            for peer in peers {
                if *peer != uuid {
                    if let Some(peer) = state.get_peer(*peer) {
                        let response = Response {
                            kind: Type::Message,
                            path: Some(request.path.clone()),
                            status: Status::Ok,
                            headers: Default::default(),
                            body: request.body.clone(),
                        };

                        peer.send(response).await;
                    }
                }
            }
        }

        Response {
            kind: Type::Publish,
            path: Some(request.path),
            status: Status::Ok,
            headers: Default::default(),
            body: None,
        }
    }

    async fn subscribe(&self, request: Request, uuid: uuid::Uuid) -> Response {
        let mut inner = self.inner.write().await;

        let peers = inner.entry(request.path.clone()).or_default();
        peers.push(uuid);

        Response {
            kind: Type::Subscribe,
            path: Some(request.path),
            status: Status::Ok,
            headers: Default::default(),
            body: None,
        }
    }

    async fn unsubscribe(&self, request: Request, uuid: uuid::Uuid) -> Response {
        let mut inner = self.inner.write().await;

        if let Some(peers) = inner.get_mut(&request.path) {
            peers.retain(|peer| *peer != uuid);
        }

        Response {
            kind: Type::Unsubscribe,
            path: Some(request.path),
            status: Status::Ok,
            headers: Default::default(),
            body: None,
        }
    }

    fn generic(&self, request: Request) -> Response {
        Response {
            kind: Type::Generic,
            path: Some(request.path),
            status: Status::Ok,
            headers: Default::default(),
            body: None,
        }
    }
}
