use uuid::Uuid;
use chrono::{Utc};
use sqlx::Type;

#[derive(Debug, Type)]
#[sqlx(type_name = "permission")]
pub enum Permission{
    USER,
    ADMIN
}

pub struct User {
    pub(crate) uuid: Uuid,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) email: String,

    pub(crate) permission: Permission,

    pub(crate) timestamp: u64,
}

impl User {
    pub fn new(username: String, password: String, email: String, permission: Permission) -> Self {
        let uuid = Uuid::new_v4();

        Self {
            uuid,
            username,
            password,
            email,
            permission,
            timestamp: Utc::now().timestamp() as u64,
        }
    }
}