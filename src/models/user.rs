use uuid::Uuid;
use chrono::{Utc};
pub enum Permission{
    USER,
    ADMIN
}

pub struct User {
    uuid: Uuid,
    username: String,
    password: String,
    email: String,

    permission: Permission,

    timestamp: u64,
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