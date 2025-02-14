use crate::models::appstate::AppstateWrapper;
use crate::models::user::AuthUser;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use sqlx::Error;

#[derive(Serialize, Deserialize)]
pub struct Body {
    new_username: String,
}

/// Changes Username to new one, dependent on password confirmation
#[axum_macros::debug_handler]
pub async fn change_username(
    auth_user: Extension<AuthUser>,
    State(appstate): State<AppstateWrapper>,
    Json(body): Json<Body>
) -> Result<StatusCode, (StatusCode, &'static str)> {
    let user = auth_user.0.0;
    let appstate = appstate.0;

    // update new username in db
    let conn = &appstate.db_pool;
    let query = r"UPDATE users SET username = $1 WHERE uuid = $2";
    let query_result = sqlx::query(query)
        .bind(body.new_username)
        .bind(user.uuid.to_string())
        .execute(conn.as_ref())
        .await;

    if let Err(e) = query_result {
        return match e {
            Error::Database(db_err) => {
                if db_err.is_unique_violation() {
                    Err((StatusCode::BAD_REQUEST, "Username is already taken"))
                } else {
                    Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to update in db"))
                }
            }
            _ => Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to update in db"))
        }
    }

    Ok(StatusCode::OK)
}