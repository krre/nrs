use axum::{
    extract::{self, State},
    http::StatusCode,
    response::IntoResponse,
};

use serde::Deserialize;
use sqlx::PgPool;

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

pub async fn create_user(
    State(pool): State<PgPool>,
    extract::Json(payload): extract::Json<CreateUser>,
) -> Result<impl IntoResponse, StatusCode> {
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

    println!("id {}", user.id);

    Ok(StatusCode::CREATED)
}
