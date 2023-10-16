use std::{
    ops::Add,
    time::{Duration, SystemTime},
};

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::types::time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    user_id: i64,
    exp: usize,
}

pub fn create_token(user_id: i64, email: &str, secret: &str) -> String {
    let from_now = Duration::from_secs(3600 * 24 * 365 * 10); // 10 years
    let expired_future_time = SystemTime::now().add(from_now);
    let exp = OffsetDateTime::from(expired_future_time);

    let claims = Claims {
        sub: String::from(email),
        exp: exp.unix_timestamp() as usize,
        user_id,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}

pub fn user_id(token: &str, secret: &str) -> i64 {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .unwrap()
    .claims
    .user_id
}
