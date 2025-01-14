use crate::models::appstate::Appstate;
use crate::util::jwt::generate::Claims;
use crate::util::jwt::validate::valid_claims;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use axum::Extension;
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::sync::Arc;
use tower_cookies::Cookies;

pub async fn auth(
    extension: Extension<Arc<Appstate>>, /* State(appstate): State<Arc<Appstate>> */
    cookies: Cookies,
    req: Request,
    next: Next
) -> Result<Response, StatusCode> {
    let appstate = extension.0;
    let token = cookies.get("token").ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // decode token
    let secret = &appstate.jwt_secret;
    let token_data = decode::<Claims>(
        token.value(),
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default()
    ).map_err(|_| StatusCode::UNAUTHORIZED)?;

    // validate token
    if !valid_claims(token_data.claims, &appstate.db_pool).await {
        return Err(StatusCode::UNAUTHORIZED)
    }

    let response = next.run(req).await;
    Ok(response)
}