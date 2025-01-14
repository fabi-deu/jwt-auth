use crate::util::jwt::generate::Claims;
use chrono::Utc;
use sqlx::{Pool, Postgres, Row};
use std::sync::Arc;

pub(crate) async fn valid_claims(claims: Claims, conn: &Arc<Pool<Postgres>>) -> bool {
    // check for timestamps
    if claims.exp < Utc::now().timestamp() as usize {
        return false
    }

    // get tokenid from db and compare
    let query = r"SELECT tokenid FROM users WHERE uuid = $1";
    let row = sqlx::query(query)
        .bind(claims.sub.to_string())
        .fetch_one(conn.as_ref())
        .await
        .unwrap();

    let tokenid: String = row.try_get("tokenid").unwrap();

    tokenid == claims.tokenid.to_string()
}