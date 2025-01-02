// 1. body parsing
// 2. validation + hashing
// 3. add user to db
// 4. sending email -> email doesn't exist -> delete user

use std::sync::Arc;
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use crate::{
    models::{
        appstate::Appstate,
        user::*,
    },
    util::validation,
};

#[derive(Serialize, Deserialize)]
pub struct Body {
    username: String,
    email: String,
    password: String,
}

pub async fn new(
    State(appstate): State<Arc<Appstate>>,
    Json(body): Json<Body>,
) -> (StatusCode, String) {
    // validate username & password
    match validation::username(&body.username) {
        (true, _) => {},
        (false, e) => {
            return (StatusCode::BAD_REQUEST, format!("Username is not valid: {}", e))
        }
    }

    match validation::password(&body.password) {
        (true, _) => {}
        (false, e) => {
            return (StatusCode::BAD_REQUEST, format!("Password is not valid: {}", e))
        }
    }

    // hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon = Argon2::default();
    let hashed_password = match argon.hash_password(body.password.as_ref(), &salt) {
        Ok(o) => o.to_string(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password".to_string())
    };

    // construct user model
    let user = User::new(
        body.username,
        hashed_password,
        body.email,
        Permission::USER /* Hard coded user permission, get admin rights other ways */
    );

    // write user to db
    let conn = &appstate.db_pool;
    let raw_query =
        r"INSERT INTO users (uuid, username, email, password, permission) VALUES ($1, $2, $3, $4, $5)";

    let query = sqlx::query(raw_query)
        .bind(&user.uuid.to_string())
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password)
        .bind(&user.permission)
        .execute(conn.as_ref()).await;

    match query {
        Ok(_) => {}
        Err(e) => return {
            eprintln!("DB WRITE: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to insert user into db".to_string())
        }
    }

    // TODO! send user email to validate or delete user

    (StatusCode::CREATED, "".to_string())
}

