use std::sync::Arc;

use axum::{
    extract::{self, State},
    Extension, Json,
};

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use validator::Validate;

use crate::{api::Error, api::Result};
use crate::{
    api::{
        extract::{AuthUser, ValidPayload},
        router::JwtExt,
    },
    core::jwt,
};

#[derive(Deserialize, Validate)]
pub struct CreateUser {
    #[validate(length(min = 1))]
    login: String,
    #[validate(length(min = 1))]
    full_name: String,
    #[validate(email)]
    email: String,
    #[validate(length(min = 1))]
    password: String,
}

#[derive(Deserialize, Validate)]
pub struct UpdateUser {
    #[validate(length(min = 1))]
    full_name: String,
}

#[derive(Deserialize, Validate)]
pub struct ChangePassword {
    #[validate(length(min = 1))]
    old_password: String,
    #[validate(length(min = 1))]
    new_password: String,
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

pub async fn create(
    State(pool): State<PgPool>,
    jwt_ext: Extension<Arc<JwtExt>>,
    ValidPayload(payload): ValidPayload<CreateUser>,
) -> Result<Json<CreateUserResponse>> {
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

pub async fn update(
    State(pool): State<PgPool>,
    AuthUser(user_id): AuthUser,
    ValidPayload(payload): ValidPayload<UpdateUser>,
) -> Result<()> {
    sqlx::query!(
        "UPDATE users SET full_name = $1, updated_at = current_timestamp WHERE id = $2",
        payload.full_name,
        user_id as i32
    )
    .execute(&pool)
    .await
    .map_err(|e| Error::InternalServerError(format!("database error: {}", e)))?;

    Ok(())
}

pub async fn change_password(
    State(pool): State<PgPool>,
    AuthUser(user_id): AuthUser,
    ValidPayload(payload): ValidPayload<ChangePassword>,
) -> Result<()> {
    sqlx::query!(
        "UPDATE users SET password = $1, updated_at = current_timestamp WHERE id = $2",
        payload.new_password,
        user_id as i32
    )
    .execute(&pool)
    .await
    .map_err(|e| Error::InternalServerError(format!("database error: {}", e)))?;

    Ok(())
}

pub async fn login(
    State(pool): State<PgPool>,
    jwt_ext: Extension<Arc<JwtExt>>,
    payload: extract::Json<LoginUser>,
) -> Result<Json<CreateUserResponse>> {
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

pub async fn get(
    State(pool): State<PgPool>,
    AuthUser(user_id): AuthUser,
) -> Result<Json<UserProfile>> {
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

pub async fn delete(State(pool): State<PgPool>, AuthUser(user_id): AuthUser) -> Result<()> {
    sqlx::query!("DELETE FROM users WHERE id = $1", user_id as i32,)
        .execute(&pool)
        .await
        .map_err(|e| Error::InternalServerError(format!("database error: {}", e)))?;

    Ok(())
}
