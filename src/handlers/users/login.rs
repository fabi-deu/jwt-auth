use crate::models::appstate::AppstateWrapper;
use crate::models::user::User;
use crate::util::jwt::claims::Claims;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::PrivateCookieJar;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Body {
    pub username: String,
    pub password: String,
}

pub async fn login(
    State(appstate): State<AppstateWrapper>,
    jar: PrivateCookieJar,
    Json(body): Json<Body>
) -> Result<(StatusCode, PrivateCookieJar, String), (StatusCode, &'static str)> {
    let appstate = appstate.0;

    // get user from db
    let user = match User::from_username(&body.username, &appstate).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err((StatusCode::BAD_REQUEST, "User does not exist")),
        _ => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to get user from db"))
    };

    // compare passwords
    match user.compare_passwords(body.password) {
        Ok(o) => {
            if !o {
                return Err((StatusCode::UNAUTHORIZED, "Wrong password"))
            }
        },
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to compare passwords"))
    }

    // generate token
    let token = match Claims::generate_jwt(&appstate.jwt_secret, &user) {
        Ok(o) => o,
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate jwt"))
    };

    // set cookies
    let mut cookie = Cookie::new("token", token.clone());
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Strict);

    let jar = jar.add(cookie);

    Ok((StatusCode::OK, jar, token))
}