use std::sync::Arc;
use axum::Extension;
use axum::extract::State;
use axum::http::StatusCode;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use crate::models::appstate::Appstate;
use crate::models::user::AuthUser;
use crate::util::jwt::claims::Claims;

#[axum_macros::debug_handler]
pub async fn refresh_token(
    auth_user: Extension<AuthUser>,
    jar: CookieJar,
    State(appstate): State<Arc<Appstate>>,
) -> Result<(StatusCode, CookieJar), (StatusCode, String)> {
    let user = auth_user.0.0;

    // generate new token
    let new_token = match Claims::generate_jwt(&appstate.jwt_secret, &user) {
        Ok(o) => o,
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate new token".to_string()))
    };

    // set new token in cookies
    let jar = jar.add(Cookie::new("token", new_token));

    Ok((StatusCode::OK, jar))
}