use crate::models::user::User;
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Claims {
    pub(crate) sub: Uuid,
    pub(crate) tokenid: Uuid,
    pub(crate) iat: usize,
    pub(crate) exp: usize,
}


pub(crate) fn generate(jwt_secret: &String, user: &User) -> jsonwebtoken::errors::Result<String> {
    let claims = Claims {
        sub: user.uuid,
        tokenid: user.tokenid,
        iat: Utc::now().timestamp() as usize,
        exp: Utc::now().timestamp() as usize + 60*60*24*365, /* 1 year */
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_ref()))
}