use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{password_hash, Argon2, PasswordHasher};

pub async fn hash_password(password: String) -> Result<String, password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hashed = argon2.hash_password(password.as_ref(), &salt)?;

    Ok(hashed.to_string())
}