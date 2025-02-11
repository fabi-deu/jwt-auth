use crate::models::appstate::AppstateWrapper;
use crate::models::user::{AuthUser, User};
use crate::util::jwt::claims::Claims;
use axum::extract::Request;
use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use axum::Extension;
use axum::http::header::AUTHORIZATION;
use axum_extra::extract::PrivateCookieJar;
use jsonwebtoken::{decode, DecodingKey, Validation};


#[axum_macros::debug_middleware]
pub async fn auth(
    Extension(appstate): Extension<AppstateWrapper>,
    mut req: Request,
    next: Next
) -> Result<Response, StatusCode> {
    let appstate = appstate.0;
    // get token
    let headers = req.headers();

    // get auth header
    let auth_header = headers.get(AUTHORIZATION);
    println!("{:?}", auth_header);

    // get cookie jar
    let jar = PrivateCookieJar::from_headers(headers, appstate.cookie_secret.clone());
    let cookie = jar.get("token");
    /*
    // determine the token
    let token = match (auth_header, cookie) {
        (Some(x), _) => {
            // remove the "Bearer: "
            x.to_str().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .strip_prefix("Bearer ").ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        }, // Bearer token takes priority over cookies
        (None, Some(y)) => &y.to_string(),
        _ => return Err(StatusCode::UNAUTHORIZED)
    };*/


    let token = jar.get("token")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    println!("a");

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