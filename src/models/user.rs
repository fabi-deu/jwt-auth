use argon2::{Argon2, PasswordHash, PasswordVerifier};
use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::{Row, Type};
use std::error::Error;
use std::future::{ready, Future};
use jsonwebtoken::{decode, DecodingKey, Validation};
use uuid::Uuid;
use crate::models::appstate::Appstate;
use crate::util::jwt::claims::Claims;

#[derive(Debug, Type, Clone, Serialize, Deserialize)]
#[sqlx(type_name = "permission")]
pub enum Permission{
    USER,
    ADMIN
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub(crate) uuid: Uuid,
    pub(crate) username: String,
    password: String,
    pub(crate) email: String,

    pub(crate) permission: Permission,
    pub(crate) tokenid: Uuid,
    pub(crate) timestamp: usize,
}

// for passing user data to next handler with auth middleware
#[derive(Clone)]
pub struct AuthUser(pub User);


impl User {
    pub fn new(username: String, password: String, email: String, permission: Permission) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            username,
            password,
            email,
            permission,
            tokenid: Uuid::new_v4(),
            timestamp: Utc::now().timestamp() as usize,
        }
    }
    /// Maps PgRow to User
    pub fn from_pg_row(row: PgRow) -> Result<User, Box<dyn Error>> {
        Ok(User{
            uuid: Uuid::parse_str(row.try_get("uuid")?)?,
            username: row.try_get("username")?,
            password: row.try_get("password")?,
            email: row.try_get("email")?,
            permission: row.try_get("permission")?,
            tokenid: Uuid::parse_str(row.try_get("tokenid")?)?,
            timestamp: row.try_get::<i64, _>("timestamp")? as usize,
        })
    }
    /// Compares hashed password from self with un-hashed attempt
    /// Returns false when error
    pub fn compare_passwords(&self, attempt: String) -> Result<bool, argon2::password_hash::Error> {
        let parsed_hash = PasswordHash::new(&self.password)?;
        let argon2 = Argon2::default();
        Ok(argon2.verify_password(attempt.as_bytes(), &parsed_hash).is_ok())
    }

    pub async fn from_token(token: String, appstate: &Appstate) -> Result<User, StatusCode> {
        // decode token
        let secret = &appstate.jwt_secret;
        let token_data = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default()
        ).map_err(|_| StatusCode::UNAUTHORIZED)?;

        // validate claims and get user model
        let claims = token_data.claims;
        let user = match claims.validate_claims(&appstate).await {
            Ok(o) => {
                match o {
                    Some(u) => u,
                    None => {
                        return Err(StatusCode::UNAUTHORIZED)
                    }
                }
            },
            Err(_) => {
                return Err( StatusCode::INTERNAL_SERVER_ERROR )
            }
        };
        Ok(user)
    }

    pub async fn from_username(username: &str, appstate: &Appstate) -> Result<Option<User>, Box<dyn Error>> {
        // get user from db
        let conn = &appstate.db_pool;
        let query_result = sqlx::query("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(conn.as_ref())
            .await?;


        match query_result {
            Some(row) => Ok(Some(User::from_pg_row(row)?)),
            None => Ok(None),
        }
    }

    pub async fn write_to_db(&self, appstate: &Appstate) -> Result<(), sqlx::Error> {
        let conn = &appstate.db_pool;
        let query =
            r"INSERT INTO users (uuid, username, email, password, permission, tokenid) VALUES ($1, $2, $3, $4, $5, $6)";

        let _ = sqlx::query(query)
            .bind(&self.uuid.to_string())
            .bind(&self.username)
            .bind(&self.email)
            .bind(&self.password)
            .bind(&self.permission)
            .bind(&self.tokenid.to_string())
            .execute(conn.as_ref()).await?;
        Ok(())
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync
{
    type Rejection = StatusCode;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let user = parts
            .extensions
            .get::<User>()
            .cloned()
            .map(AuthUser)
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR);

        ready(user)
    }
}