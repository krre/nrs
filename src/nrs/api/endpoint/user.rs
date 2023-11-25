use std::sync::Arc;

use axum::{
    extract::{self, State},
    Extension, Json,
};

use sqlx::PgPool;

use crate::{
    api::Error,
    api::{self, Result},
};
use crate::{
    api::{
        extract::{AuthUser, ValidPayload},
        router::JwtExt,
    },
    core::jwt,
};

mod request {
    use serde::Deserialize;
    use validator::Validate;

    #[derive(Deserialize, Validate)]
    pub struct Create {
        #[validate(length(min = 1))]
        pub login: String,
        #[validate(length(min = 1))]
        pub full_name: String,
        #[validate(email)]
        pub email: String,
        #[validate(length(min = 1))]
        pub password: String,
    }

    #[derive(Deserialize, Validate)]
    pub struct Update {
        #[validate(length(min = 1))]
        pub full_name: String,
    }

    #[derive(Deserialize, Validate)]
    pub struct ChangePassword {
        #[validate(length(min = 1))]
        pub old_password: String,
        #[validate(length(min = 1))]
        pub new_password: String,
    }

    #[derive(Deserialize, Validate)]
    pub struct Login {
        #[validate(email)]
        pub email: String,
        #[validate(length(min = 1))]
        pub password: String,
    }
}

mod response {
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct Create {
        pub token: String,
    }

    #[derive(Serialize)]
    pub struct Profile {
        pub login: String,
        pub full_name: String,
        pub email: String,
    }
}

pub async fn create(
    State(pool): State<PgPool>,
    jwt_ext: Extension<Arc<JwtExt>>,
    ValidPayload(payload): ValidPayload<request::Create>,
) -> Result<Json<response::Create>> {
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
            return Ok(Json(response::Create { token }));
        }
        Err(error) => match error {
            sqlx::Error::Database(database_error) => {
                if database_error.is_unique_violation() {
                    return Err(Error::Conflict);
                }
            }
            _ => return Err(api::error::Error::DatabaseError(error)),
        },
    }

    Err(Error::InternalServerError("Unexpected error".to_string()))
}

pub async fn update(
    State(pool): State<PgPool>,
    AuthUser(user_id): AuthUser,
    ValidPayload(payload): ValidPayload<request::Update>,
) -> Result<()> {
    sqlx::query!(
        "UPDATE users SET full_name = $1, updated_at = current_timestamp WHERE id = $2",
        payload.full_name,
        user_id as i32
    )
    .execute(&pool)
    .await?;

    Ok(())
}

pub async fn change_password(
    State(pool): State<PgPool>,
    AuthUser(user_id): AuthUser,
    ValidPayload(payload): ValidPayload<request::ChangePassword>,
) -> Result<()> {
    struct User {
        password: String,
    }

    let user = sqlx::query_as!(
        User,
        "SELECT password FROM users WHERE id = $1",
        user_id as i32
    )
    .fetch_one(&pool)
    .await?;

    if user.password != payload.old_password {
        return Err(Error::BadRequest("invalid password".to_string()));
    }

    sqlx::query!(
        "UPDATE users SET password = $1, updated_at = current_timestamp WHERE id = $2",
        payload.new_password,
        user_id as i32
    )
    .execute(&pool)
    .await?;

    Ok(())
}

pub async fn login(
    State(pool): State<PgPool>,
    jwt_ext: Extension<Arc<JwtExt>>,
    payload: extract::Json<request::Login>,
) -> Result<Json<response::Create>> {
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

            return Ok(Json(response::Create { token }));
        }
        Err(error) => match error {
            sqlx::Error::RowNotFound => {
                return Err(Error::NotFound(format!("email `{}` not found", error)));
            }
            _ => {
                return Err(api::error::Error::DatabaseError(error));
            }
        },
    }
}

pub async fn get(
    State(pool): State<PgPool>,
    AuthUser(user_id): AuthUser,
) -> Result<Json<response::Profile>> {
    let user = sqlx::query_as!(
        response::Profile,
        "SELECT login, full_name, email FROM users WHERE id = $1",
        user_id as i32,
    )
    .fetch_one(&pool)
    .await?;

    Ok(Json(user))
}

pub async fn delete(State(pool): State<PgPool>, AuthUser(user_id): AuthUser) -> Result<()> {
    sqlx::query!("DELETE FROM users WHERE id = $1", user_id as i32,)
        .execute(&pool)
        .await?;

    Ok(())
}
