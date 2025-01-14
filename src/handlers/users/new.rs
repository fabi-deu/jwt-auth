// 1. body parsing
// 2. validation + hashing
// 3. add user to db
// 4. sending email -> email doesn't exist -> delete user

use crate::{
    models::{
        appstate::Appstate,
        user::*,
    },
    util::validation,
};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use axum::response::IntoResponse;
use axum::{extract::State, http::StatusCode, Extension, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use axum::extract::Request;
use tower_cookies::{Cookie, Cookies};
use crate::util::jwt::generate::generate;

#[derive(Serialize, Deserialize)]
pub struct Body {
    username: String,
    email: String,
    password: String,
}

#[axum_macros::debug_handler]
pub async fn new(
    State(appstate): State<Arc<Appstate>>,
    cookies: Cookies,
    Json(body): Json<Body>,
) -> impl IntoResponse {
    // validate username & password
    match validation::username(&body.username) {
        (true, _) => {},
        (false, e) => {
                return ( StatusCode::BAD_REQUEST, format!("Username is not valid: {}", e) )
        }
    }

    match validation::password(&body.password) {
        (true, _) => {}
        (false, e) => {
            return ( StatusCode::BAD_REQUEST, format!("Password is not valid: {}", e) )
        }
    }

    // hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon = Argon2::default();
    let hashed_password = match argon.hash_password(body.password.as_ref(), &salt) {
        Ok(o) => o.to_string(),
        Err(_) => return ( StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password".to_string() )
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
    let token = generate(&appstate.jwt_secret, &user);
    let Ok(token) = token else {
        println!("Failed to generate jwt");
        return ( StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate jwt".to_string() )
    };

    // write user to db
    let conn = &appstate.db_pool;
    let query =
        r"INSERT INTO users (uuid, username, email, password, permission, tokenid) VALUES ($1, $2, $3, $4, $5, $6)";


    let Ok(_) = sqlx::query(query)
        .bind(&user.uuid.to_string())
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password)
        .bind(&user.permission)
        .bind(&user.tokenid.to_string())
        .execute(conn.as_ref()).await
        else {
            return ( StatusCode::INTERNAL_SERVER_ERROR, "Failed to insert user into db".to_string() )
        };

    // set cookie
    cookies.add(Cookie::new("token", token));
    ( StatusCode::CREATED, String::new() )
}