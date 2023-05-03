#![allow(clippy::needless_return)]

use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};

use axum_client_ip::InsecureClientIp;

use super::stores::{MemoryStore, RedisStore, Store};

#[derive(Clone)]
pub struct Options {
    pub max: usize,
}

type ShareableState<T> = Arc<RwLock<T>>;

#[derive(Clone)]
pub struct RateLimiter<T>
where
    T: Store + Clone,
{
    pub options: Options,
    pub store: ShareableState<T>,
}

impl<T> RateLimiter<T>
where
    T: Store + Clone,
{
    pub fn new(options: Options, store: T) -> Self {
        let store = Arc::new(RwLock::new(store));
        return Self { options, store };
    }

    pub async fn middleware<B>(
        State(state): State<Self>,
        InsecureClientIp(ip_addr): InsecureClientIp,
        req: Request<B>,
        next: Next<B>,
    ) -> Response {
        {
            let ip = ip_addr.to_string();

            let current: usize = {
                let store = state.store.read().await;
                store.get(&ip).await.unwrap().unwrap_or(0)
            };

            let latest = current + 1;

            state.store.write().await.update(&ip, latest).await.unwrap();

            if latest > state.options.max {
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    format!("{} max requests per minute", state.options.max),
                )
                    .into_response();
            }
        }

        return next.run(req).await;
    }
}
