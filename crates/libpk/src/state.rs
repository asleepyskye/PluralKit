#[derive(serde::Serialize, serde::Deserialize, Clone, Default)]
pub struct ShardState {
    pub cluster_id: Option<i32>,
    pub shard_id: i32,
    pub up: bool,
    /// milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency: Option<i32>,
    /// unix timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_heartbeat: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_connection: Option<i32>,
}
