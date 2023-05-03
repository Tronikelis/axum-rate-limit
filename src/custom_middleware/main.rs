use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_client_ip::InsecureClientIp;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::stores::Store;

#[derive(Clone)]
pub struct Options {
    pub max: usize,
    pub per_min: usize,
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
        let mut latest: usize = 0;

        {
            let ip = ip_addr.to_string();

            let current: usize = {
                let mut store = state.store.write().await;
                store.get(&ip).await.unwrap().unwrap_or(0)
            };

            latest = current + 1;

            state.store.write().await.update(&ip, latest).await.unwrap();

            if latest > state.options.max {
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    format!(
                        "{} max requests per {}",
                        state.options.max, state.options.per_min
                    ),
                )
                    .into_response();
            }
        }

        let mut response = next.run(req).await;

        let headers_mut = response.headers_mut();
        headers_mut.insert("x-rate-limit-current", latest.into());
        headers_mut.insert("x-rate-limit-max", state.options.max.into());

        return response;
    }
}
