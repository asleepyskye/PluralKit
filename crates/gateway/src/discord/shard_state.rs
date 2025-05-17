use metrics::{counter, gauge};
use tracing::info;
use twilight_gateway::{Event, Latency};

use libpk::state::ShardState;

#[derive(Clone)]
pub struct ShardStateManager {
    manager_url: String,
    cluster_id: u32,
}

pub fn new(manager_url: String, cluster_id: u32) -> ShardStateManager {
    ShardStateManager {
        manager_url,
        cluster_id,
    }
}

impl ShardStateManager {
    pub async fn handle_event(&self, shard_id: u32, event: Event) -> anyhow::Result<()> {
        match event {
            Event::Ready(_) => self.ready_or_resumed(shard_id, false).await,
            Event::Resumed => self.ready_or_resumed(shard_id, true).await,
            _ => Ok(()),
        }
    }

    async fn patch_shard(&self, info: ShardState) -> anyhow::Result<()> {
        let client = reqwest::Client::new();
        client
            .patch(format!("http://{}/shard/status", self.manager_url))
            .body(serde_json::to_string(&info).expect("could not serialize shard"))
            .send()
            .await?;
        Ok(())
    }

    async fn ready_or_resumed(&self, shard_id: u32, resumed: bool) -> anyhow::Result<()> {
        info!(
            "shard {} {}",
            shard_id,
            if resumed { "resumed" } else { "ready" }
        );
        counter!(
            "pluralkit_gateway_shard_reconnect",
            "shard_id" => shard_id.to_string(),
            "resumed" => resumed.to_string(),
        )
        .increment(1);
        gauge!("pluralkit_gateway_shard_up").increment(1);

        let info = ShardState {
            shard_id: shard_id as i32,
            cluster_id: Some(self.cluster_id as i32),
            up: true,
            last_connection: Some(chrono::offset::Utc::now().timestamp() as i32),
            last_heartbeat: None,
            latency: None,
        };
        self.patch_shard(info).await?;
        Ok(())
    }

    pub async fn socket_closed(&self, shard_id: u32) -> anyhow::Result<()> {
        gauge!("pluralkit_gateway_shard_up").decrement(1);
        counter!(
            "pluralkit_gateway_shard_disconnect",
            "shard_id" => shard_id.to_string(),
        )
        .increment(1);
        let info = ShardState {
            shard_id: shard_id as i32,
            cluster_id: Some(self.cluster_id as i32),
            up: false,
            last_connection: None,
            last_heartbeat: None,
            latency: None,
        };
        self.patch_shard(info).await?;
        Ok(())
    }

    pub async fn heartbeated(&self, shard_id: u32, latency: &Latency) -> anyhow::Result<()> {
        let shard_latency = latency
            .recent()
            .first()
            .map_or_else(|| 0, |d| d.as_millis()) as i32;
        let info = ShardState {
            shard_id: shard_id as i32,
            cluster_id: Some(self.cluster_id as i32),
            up: true,
            last_connection: None,
            last_heartbeat: Some(chrono::offset::Utc::now().timestamp() as i32),
            latency: Some(shard_latency),
        };
        gauge!("pluralkit_gateway_shard_latency", "shard_id" => shard_id.to_string())
            .set(shard_latency);
        self.patch_shard(info).await?;
        Ok(())
    }
}
