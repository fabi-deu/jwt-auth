use crate::models::appstate::AppstateWrapper;
use crate::models::user::{AuthUser, User};
use axum::extract::Request;
use axum::http::header::AUTHORIZATION;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use axum::Extension;
use axum_extra::extract::PrivateCookieJar;


#[axum_macros::debug_middleware]
pub async fn auth(
    Extension(appstate): Extension<AppstateWrapper>,
    mut req: Request,
    next: Next
) -> Result<Response, StatusCode> {
    let appstate = appstate.0;

    // EXTRACTING TOKEN
    let headers = req.headers();

    let auth_header = headers.get(AUTHORIZATION);
     // get cookie jar as user could use both Bearer or cookie
    let jar = PrivateCookieJar::from_headers(headers, appstate.cookie_secret.clone());
    let cookie = jar.get("token");

     // determine the token
    let token = match (auth_header, cookie) {
        (Some(x), _) => {
            // remove the "Bearer: "
            x.to_str().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .strip_prefix("Bearer ").ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        }, // Bearer token takes priority over cookies
        (None, Some(y)) => &y.value().to_string(),
        _ => return Err(StatusCode::UNAUTHORIZED)
    };


    // get user from token
    let user = match User::from_token(token.to_string(), &appstate).await {
        Ok(o) => o,
        _ => return Err(StatusCode::UNAUTHORIZED)
    };


    // pass user to next handler
    req.extensions_mut().insert(AuthUser(user));
    let response = next.run(req).await;
    Ok(response)
}