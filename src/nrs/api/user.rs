use std::sync::Arc;

use axum::{
    extract::{self, State},
    http::StatusCode,
    Extension, Json,
};

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::core::jwt;
use crate::core::router::JwtExt;

#[derive(Deserialize)]
pub struct CreateUser {
    sign: String,
    name: String,
    email: String,
    password: String,
}

struct User {
    id: i32,
}
#[derive(Serialize)]
pub struct CreateUserResponse {
    token: String,
}

pub async fn create_user(
    State(pool): State<PgPool>,
    jwt_ext: Extension<Arc<JwtExt>>,
    extract::Json(payload): extract::Json<CreateUser>,
) -> Result<Json<CreateUserResponse>, StatusCode> {
    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (sign, name, email, password) values ($1, $2, $3, $4) RETURNING id",
        payload.sign,
        payload.name,
        payload.email,
        payload.password
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let token = jwt::create_token(user.id as i64, &payload.email, &jwt_ext.secret);

    if let Ok(token) = token {
        Ok(Json(CreateUserResponse { token }))
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
