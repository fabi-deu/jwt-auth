use crate::models::user::User;
use crate::util::jwt::generate::Claims;
use chrono::Utc;
use sqlx::{Pool, Postgres};
use std::error::Error;
use std::sync::Arc;

/// Validates Claims
/// Returns Some(User) when valid
pub(crate) async fn valid_claims(
    claims: Claims, conn: &Arc<Pool<Postgres>>
) -> Result<Option<User>, Box<dyn Error>> {

    // check for timestamps
    if claims.exp < Utc::now().timestamp() as usize {
        return Ok(None)
    }

    // get user from db
    // use query_as macro instead (can't figure it out)
    let query = r"SELECT * FROM users WHERE uuid = $1";
    let row = sqlx::query(query)
        .bind(&claims.sub.to_string())
        .fetch_one(conn.as_ref())
        .await?;

    let user = User::from_pg_row(row)?;

    // compare ids
    if user.tokenid != claims.tokenid {
        return Ok(None)
    }

    Ok(Some(user))
}