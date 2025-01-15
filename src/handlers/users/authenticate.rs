use crate::models::appstate::Appstate;
use crate::util::jwt::generate::Claims;
use crate::util::jwt::validate::valid_claims;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use axum::Extension;
use axum_extra::extract::CookieJar;
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::sync::Arc;
use crate::models::user::AuthUser;

#[axum_macros::debug_middleware]
pub async fn auth(
    appstate: Extension<Arc<Appstate>>,
    jar: CookieJar,
    mut req: Request,
    next: Next
) -> Result<Response, StatusCode> {
    let token = jar.get("token")
        .ok_or(StatusCode::BAD_REQUEST)?;

    println!("{}", token.value());

    // decode token
    let secret = &appstate.jwt_secret;
    let token_data = decode::<Claims>(
        token.value(),
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default()
    ).map_err(|_| StatusCode::UNAUTHORIZED)?;

    // validate claims and get user model
    let user = match valid_claims(token_data.claims, &appstate.db_pool).await {
        Ok(o) => {
            if o.1 != StatusCode::OK { return Err( StatusCode::UNAUTHORIZED ) }
            o.0.unwrap() // unwrapping is ok here as we know its Some() from the StatusCode
        }
        Err(_) => return Err( StatusCode::INTERNAL_SERVER_ERROR )
    };


    // pass user to next handler
    req.extensions_mut().insert(AuthUser(user));
    let response = next.run(req).await;
    Ok(response)
}