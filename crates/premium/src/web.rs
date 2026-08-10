use askama::Template;
use serde::Serialize;

use crate::auth::AuthState;
use crate::payments::SubscriptionInfo;

macro_rules! render {
    ($stuff:expr) => {{ axum::Json($stuff).into_response() }};
}

pub(crate) use render;

pub fn message(message: String, session: Option<AuthState>) -> Index {
    Index {
        base_url: libpk::config.premium().base_url.clone(),
        session,
        show_login_form: false,
        message: Some(message),
        subscriptions: vec![],
    }
}

#[derive(Serialize)]
pub struct Index {
    pub base_url: String,
    pub session: Option<AuthState>,
    pub show_login_form: bool,
    pub message: Option<String>,
    pub subscriptions: Vec<SubscriptionInfo>,
}

#[derive(Serialize)]
pub struct Cancel {
    pub csrf_token: String,
    pub subscription: crate::payments::SubscriptionInfo,
}

#[derive(Serialize)]
pub struct Exchange {
    pub session_token: String,
    pub expires_in: i64,
}

#[derive(Serialize)]
pub struct SessionInfo {
    pub email: String,
    pub csrf_token: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct System {
    pub hid: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Subscription {
    pub id: String,
    pub status: String,
    pub next_renewal: String,
    pub system: Option<System>,
    pub is_lifetime: bool,
    pub is_cancellable: bool,
}

impl From<&SubscriptionInfo> for Subscription {
    fn from(info: &SubscriptionInfo) -> Self {
        Self {
            id: info.subscription_id().to_string(),
            status: info.status(),
            next_renewal: info.next_renewal(),
            system: info.system(),
            is_lifetime: info.is_lifetime(),
            is_cancellable: info.is_cancellable(),
        }
    }
}
