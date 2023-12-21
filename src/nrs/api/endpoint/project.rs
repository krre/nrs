pub(crate) mod router {
    use axum::routing::{self, delete, get, post, put};
    use sqlx::{Pool, Postgres};

    use super::handler;

    pub fn new(pool: &Pool<Postgres>) -> routing::Router {
        routing::Router::new()
            .route("/", get(handler::get_all))
            .route("/:id", get(handler::get_one))
            .route("/", post(handler::create))
            .route("/:id", put(handler::update))
            .route("/:id", delete(handler::delete))
            .with_state(pool.clone())
    }
}

mod handler {
    use axum::extract::Path;
    use axum::{extract::State, Json};
    use sqlx::PgPool;

    use crate::api::extract::{AuthUser, ValidPayload};
    use crate::api::Result;

    mod request {
        use serde::Deserialize;
        use validator::Validate;

        #[derive(Deserialize, Validate)]
        pub struct Create {
            #[validate(length(min = 1))]
            pub name: String,
            pub template: i16,
            pub description: String,
        }

        #[derive(Deserialize, Validate)]
        pub struct Update {
            #[validate(length(min = 1))]
            pub name: String,
            pub description: String,
        }
    }

    mod response {
        use serde::Serialize;

        #[derive(Serialize)]
        pub struct Create {
            pub id: i32,
        }

        #[derive(Serialize)]
        pub struct Project {
            pub id: i32,
            pub name: String,
            pub template: i16,
            pub description: String,
            pub created_at: chrono::DateTime<chrono::Utc>,
            pub updated_at: chrono::DateTime<chrono::Utc>,
        }
    }

    pub async fn create(
        State(pool): State<PgPool>,
        AuthUser(user_id): AuthUser,
        ValidPayload(payload): ValidPayload<request::Create>,
    ) -> Result<Json<response::Create>> {
        struct Project {
            id: i32,
        }

        let project = sqlx::query_as!(
            Project,
            "INSERT INTO projects (user_id, name, template, description) values ($1, $2, $3, $4) RETURNING id",
            user_id as i32,
            payload.name,
            payload.template,
            payload.description,
        )
        .fetch_one(&pool)
        .await?;

        Ok(Json(response::Create { id: project.id }))
    }

    pub async fn update(
        Path(id): Path<i32>,
        State(pool): State<PgPool>,
        AuthUser(user_id): AuthUser,
        ValidPayload(payload): ValidPayload<request::Update>,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE projects SET name = $1, description = $2, updated_at = current_timestamp WHERE id = $3 AND user_id = $4",
            payload.name,
            payload.description,
            id,
            user_id as i32
        )
        .execute(&pool)
        .await?;

        Ok(())
    }

    pub async fn get_all(
        State(pool): State<PgPool>,
        AuthUser(user_id): AuthUser,
    ) -> Result<Json<Vec<response::Project>>> {
        let projects = sqlx::query_as!(
            response::Project,
            "SELECT id, name, template, description, created_at, updated_at FROM projects
            WHERE user_id = $1
            ORDER BY updated_at DESC",
            user_id as i32,
        )
        .fetch_all(&pool)
        .await?;

        Ok(Json(projects))
    }

    pub async fn get_one(
        Path(id): Path<i32>,
        State(pool): State<PgPool>,
        AuthUser(user_id): AuthUser,
    ) -> Result<Json<response::Project>> {
        let project = sqlx::query_as!(
            response::Project,
            "SELECT id, name, template, description, created_at, updated_at FROM projects
            WHERE id = $1 AND user_id = $2",
            id,
            user_id as i32,
        )
        .fetch_one(&pool)
        .await?;

        Ok(Json(project))
    }

    pub async fn delete(
        Path(id): Path<i32>,
        State(pool): State<PgPool>,
        AuthUser(user_id): AuthUser,
    ) -> Result<()> {
        sqlx::query!(
            "DELETE FROM projects WHERE id = $1 AND user_id = $2",
            id,
            user_id as i32,
        )
        .execute(&pool)
        .await?;

        Ok(())
    }
}
