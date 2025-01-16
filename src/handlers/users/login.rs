use std::sync::Arc;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use crate::models::appstate::Appstate;
use crate::models::user::User;
use crate::util::jwt::generate::generate;

#[derive(Serialize, Deserialize)]
pub struct Body {
    pub username: String,
    pub password: String,
}

pub async fn login(
    State(appstate): State<Arc<Appstate>>,
    jar: CookieJar,
    Json(body): Json<Body>
) -> Result<(StatusCode, CookieJar), (StatusCode, String)> {
    // get user from db
    let conn = &appstate.db_pool;
    let query_result = sqlx::query("SELECT * FROM users WHERE username = $1")
        .bind(body.username)
        .fetch_optional(conn.as_ref())
        .await;

    let row = match query_result {
        Ok(Some(row)) => row,
        Ok(None) => return Err((StatusCode::BAD_REQUEST, "User does not exist".to_string())),
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch user from db".to_string()))
    };

    // generate token
    let user= User::from_pg_row(row)
        .ok().ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse user".to_string()))?;

    let token = match generate(&appstate.jwt_secret, &user) {
        Ok(o) => o,
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate jwt".to_string()))
    };

    // add token to cookies
    let jar = jar.add(Cookie::new("token", token));

    Ok((StatusCode::OK, jar))
}