use crate::models::appstate::AppstateWrapper;
use crate::models::user::{AuthUser, User};
use axum::extract::Request;
use axum::http::header::AUTHORIZATION;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use axum::Extension;
use axum_extra::extract::PrivateCookieJar;

/// Authentication middleware
#[axum_macros::debug_middleware]
pub async fn auth(
    Extension(appstate): Extension<AppstateWrapper>,
    mut req: Request,
    next: Next
) -> Result<Response, StatusCode> {
    let appstate = appstate.0;
    let headers = req.headers();

    // get token from both auth headers and cookies
    let auth_header = headers.get(AUTHORIZATION);
    let jar = PrivateCookieJar::from_headers(headers, appstate.cookie_secret.clone());
    let cookie = jar.get("token");

     // determine which token to use with Bearer taking priority
    let token = match (auth_header, cookie) {
        (Some(header_token), _) => {
            // remove the "Bearer: "
            header_token.to_str().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .strip_prefix("Bearer ").ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        }, // Bearer token takes priority over cookies
        (None, Some(cookie_token)) => &cookie_token.value().to_string(),
        _ => return Err(StatusCode::UNAUTHORIZED)
    };


    // get user from token
    let user = match User::from_token(token.to_string(), &appstate).await {
        Ok(o) => o,
        // could technically be something like a db error so maybe INTERNAL_SERVER_ERROR not UNAUTHORIZED?
        _ => return Err(StatusCode::UNAUTHORIZED)
    };


    // pass wrapped user to next handler
    req.extensions_mut().insert(AuthUser(user));
    let response = next.run(req).await;
    Ok(response)
}