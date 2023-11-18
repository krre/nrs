use std::sync::Arc;

use axum::{
    extract::{self, State},
    Extension, Json,
};

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::core::jwt;
use crate::core::router::JwtExt;

use super::{error::Error, extract::auth_user::AuthUser};

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

#[derive(Serialize)]
pub struct CreateUserResponse {
    token: String,
}

pub async fn create_user(
    State(pool): State<PgPool>,
    jwt_ext: Extension<Arc<JwtExt>>,
    payload: extract::Json<CreateUser>,
) -> Result<Json<CreateUserResponse>, Error> {
    struct User {
        id: i32,
    }

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
            let token = jwt::create_token(user.id as i64, &payload.email, &jwt_ext.secret)
                .map_err(|e| Error::InternalServerError(format!("cannot create token: {}", e)))?;
            return Ok(Json(CreateUserResponse { token }));
        }
        Err(error) => match error {
            sqlx::Error::Database(database_error) => {
                if database_error.is_unique_violation() {
                    return Err(Error::Conflict);
                }
            }
            _ => {
                return Err(Error::InternalServerError(format!(
                    "database error: {}",
                    error
                )));
            }
        },
    }

    Err(Error::InternalServerError("Unexpected error".to_string()))
}

pub async fn login(
    State(pool): State<PgPool>,
    jwt_ext: Extension<Arc<JwtExt>>,
    payload: extract::Json<LoginUser>,
) -> Result<Json<CreateUserResponse>, Error> {
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
                return Err(Error::Unauthorized("wrong password".to_string()));
            }

            let token = jwt::create_token(user.id as i64, &payload.email, &jwt_ext.secret)
                .map_err(|e| Error::InternalServerError(format!("cannot create token: {}", e)))?;

            return Ok(Json(CreateUserResponse { token }));
        }
        Err(error) => match error {
            sqlx::Error::RowNotFound => {
                return Err(Error::NotFound(format!("email `{}` not found", error)));
            }
            _ => {
                return Err(Error::InternalServerError(format!(
                    "database error: {}",
                    error
                )));
            }
        },
    }
}

pub async fn get_user(
    State(pool): State<PgPool>,
    AuthUser(user_id): AuthUser,
) -> Result<Json<UserProfile>, Error> {
    println!("id {}", user_id);

    let user = sqlx::query_as!(
        UserProfile,
        "SELECT login, full_name, email FROM users WHERE id = $1",
        user_id as i32,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| Error::InternalServerError(format!("database error: {}", e)))?;

    Ok(Json(user))
}
