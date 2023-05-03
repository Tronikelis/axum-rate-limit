#![allow(clippy::needless_return)]
use axum::{middleware, routing::get, Router};
use std::net::SocketAddr;

mod custom_middleware;
use custom_middleware::{
    main::{Options, RateLimiter},
    stores::{MemoryStore, RedisStore},
};

async fn root() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    let rate_limiter_options = Options {
        max: 10,
        per_min: 1,
    };

    let rate_limiter = RateLimiter::new(
        rate_limiter_options.clone(),
        RedisStore::new(
            redis::Client::open("redis://127.0.0.1").unwrap(),
            rate_limiter_options,
        ),
    );

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route_layer(middleware::from_fn_with_state(
            rate_limiter,
            RateLimiter::middleware,
        ));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
