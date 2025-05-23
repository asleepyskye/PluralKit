use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use tracing::error;

use crate::auth::AuthState;
use crate::{util::json_err, ApiContext};

pub async fn auth(State(ctx): State<ApiContext>, mut req: Request, next: Next) -> Response {
    let mut authed_system_id: Option<i32> = None;
    let mut authed_app_id: Option<i32> = None;

    // fetch user authorization
    if let Some(system_auth_header) = req
        .headers()
        .get("authorization")
        .map(|h| h.to_str().ok())
        .flatten()
        && let Some(system_id) =
            match libpk::db::repository::legacy_token_auth(&ctx.db, system_auth_header).await {
                Ok(val) => val,
                Err(err) => {
                    error!(?err, "failed to query authorization token in postgres");
                    return json_err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        r#"{"message": "500: Internal Server Error", "code": 0}"#.to_string(),
                    );
                }
            }
    {
        authed_system_id = Some(system_id);
    }

    // fetch app authorization
    // todo: actually fetch it from db
    if let Some(app_auth_header) = req
        .headers()
        .get("x-pluralkit-app")
        .map(|h| h.to_str().ok())
        .flatten()
        && let Some(config_token2) = libpk::config
            .api
            .as_ref()
            .expect("missing api config")
            .temp_token2
            .as_ref()
        // this is NOT how you validate tokens
        // but this is low abuse risk so we're keeping it for now
        && app_auth_header == config_token2
    {
        authed_app_id = Some(1);
    }

    req.extensions_mut()
        .insert(AuthState::new(authed_system_id, authed_app_id));

    next.run(req).await
}
