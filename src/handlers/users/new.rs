use crate::models::appstate::AppstateWrapper;
use crate::util::hash::hash_password;
use crate::util::jwt::claims::Claims;
use crate::{
    models::user::*,
    util::validation,
};
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::PrivateCookieJar;
use serde::{Deserialize, Serialize};
use sqlx::Error;

#[derive(Serialize, Deserialize)]
pub struct Body {
    username: String,
    email: String,
    password: String,
}
/// public handler for creating new user
#[axum_macros::debug_handler]
pub async fn new(
    State(appstate): State<AppstateWrapper>,
    jar: PrivateCookieJar,
    Json(body): Json<Body>,
) -> Result<(StatusCode, PrivateCookieJar), (StatusCode, String)> {
    let appstate = appstate.0;

    // validate username & password
    match validation::username(&body.username) {
        (true, _) => {},
        (false, e) => {
                return Err((StatusCode::BAD_REQUEST, format!("Username is not valid: {}", e)))
        }
    }

    match validation::password(&body.password) {
        (true, _) => {}
        (false, e) => {
            return Err((StatusCode::BAD_REQUEST, format!("Password is not valid: {}", e)))
        }
    }

    // hash password
    // hashing the password should be done after checking for unique username
    let hashed_password = match hash_password(body.password).await {
        Ok(hash) => hash,
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password".to_string()))
    };

    // construct user model
    let user = User::new(
        body.username,
        hashed_password,
        body.email,
        Permission::USER /* Hard coded user permission, admin rights yet to be implemented */
    );

    // TODO! send user email to validate

    // generate jwt for user
    let token = Claims::generate_jwt(&appstate.jwt_secret, &user);
    let Ok(token) = token else {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate jwt".to_string()))
    };

    // write user to db
    let query_result = user.write_to_db(&appstate);
    if let Err(e) = query_result.await {
        return match e {
            Error::Database(db_err) => {
                if db_err.is_unique_violation() {
                    Err((StatusCode::BAD_REQUEST, "Username is already taken".to_string()))
                } else {
                    Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to write to database".to_string()))
                }
            }
            _ => Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to write to database".to_string()))
        }
    }

    // set cookie
    let mut cookie = Cookie::new("token", token);
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Strict);

    let jar = jar.add(cookie);

    Ok((StatusCode::CREATED, jar))
}