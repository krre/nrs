use std::sync::Arc;

use axum::{
    extract::{self, State},
    http::StatusCode,
    Extension, Json,
};

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::error;

use crate::core::jwt;
use crate::core::router::JwtExt;

#[derive(Deserialize)]
pub struct CreateUser {
    login: String,
    full_name: String,
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct UserProfile {
    login: String,
    full_name: String,
    email: String,
}

#[derive(Deserialize)]
pub struct LoginUser {
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
        "INSERT INTO users (login, full_name, email, password) values ($1, $2, $3, $4) RETURNING id",
        payload.login,
        payload.full_name,
        payload.email,
        payload.password
    )
    .fetch_one(&pool)
    .await;

    match user {
        Ok(user) => {
            let token = jwt::create_token(user.id as i64, &payload.email, &jwt_ext.secret);

            match token {
                Ok(token) => {
                    return Ok(Json(CreateUserResponse { token }));
                }
                Err(error) => {
                    error!("cannot create token: {}", error);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
        }
        Err(error) => match error {
            sqlx::Error::Database(database_error) => {
                if database_error.is_unique_violation() {
                    return Err(StatusCode::CONFLICT);
                }
            }
            _ => {
                error!("database error: {}", error);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        },
    }

    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn login(
    State(pool): State<PgPool>,
    jwt_ext: Extension<Arc<JwtExt>>,
    extract::Json(payload): extract::Json<LoginUser>,
) -> Result<Json<CreateUserResponse>, StatusCode> {
    struct User {
        id: i32,
        password: String,
    }

    let user = sqlx::query_as!(
        User,
        "SELECT id, password FROM users WHERE email = $1",
        payload.email,
    )
    .fetch_one(&pool)
    .await;

    match user {
        Ok(user) => {
            if user.password != payload.password {
                error!("wrong password");
                return Err(StatusCode::UNAUTHORIZED);
            }

            let token = jwt::create_token(user.id as i64, &payload.email, &jwt_ext.secret);

            match token {
                Ok(token) => {
                    return Ok(Json(CreateUserResponse { token }));
                }
                Err(error) => {
                    error!("cannot create token: {}", error);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
        }
        Err(error) => match error {
            sqlx::Error::RowNotFound => {
                error!("email `{}` not found", payload.email);
                return Err(StatusCode::NOT_FOUND);
            }
            _ => {
                error!("database error: {}", error);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        },
    }
}

pub async fn get_user() -> Result<Json<UserProfile>, StatusCode> {
    Ok(Json(UserProfile {
        login: "login".to_string(),
        full_name: "full_name".to_string(),
        email: "email".to_string(),
    }))
}
