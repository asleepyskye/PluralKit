use axum::{
    Router,
    extract::{Path, State},
    routing::post,
};
use fred::prelude::RedisPool;
use twilight_model::gateway::payload::incoming::InteractionCreate;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct GatewayEnvelope {
    d: InteractionCreate,
}

async fn handle_interaction(
    State(ctx): State<InteractionsContext>,
    Path(shard_id): Path<u64>,
    body: String,
) -> axum::http::StatusCode {
    let interaction = match serde_json::from_str::<GatewayEnvelope>(&body) {
        Ok(parsed) => parsed.d,
        Err(e) => {
            tracing::error!("failed to parse interaction: {:?}", e);
            return axum::http::StatusCode::BAD_REQUEST;
        }
    };
    println!("{:?}", interaction);
    axum::http::StatusCode::OK
}

#[derive(Clone)]
pub struct InteractionsContext {
    redis: RedisPool,
}

#[libpk::main]
async fn main() -> anyhow::Result<()> {
    let config = libpk::config.interactions();
    let ctx = InteractionsContext {
        redis: libpk::db::init_redis().await?,
    };

    let app = Router::new()
        .route("/events/{shard_id}", post(handle_interaction))
        .with_state(ctx);

    let listener = tokio::net::TcpListener::bind(config.bind_addr.clone())
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
