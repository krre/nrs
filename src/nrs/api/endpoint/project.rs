pub(crate) mod router {
    use axum::routing::{self, get, post};
    use sqlx::{Pool, Postgres};

    use super::handler;

    pub fn new(pool: &Pool<Postgres>) -> routing::Router {
        routing::Router::new()
            .route("/", get(handler::get))
            .route("/", post(handler::create))
            .with_state(pool.clone())
    }
}

mod handler {
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

    pub async fn get(
        State(pool): State<PgPool>,
        AuthUser(user_id): AuthUser,
    ) -> Result<Json<Vec<response::Project>>> {
        let projects = sqlx::query_as!(
            response::Project,
            "SELECT id, name, template, description, created_at, updated_at FROM projects WHERE user_id = $1",
            user_id as i32,
        )
        .fetch_all(&pool)
        .await?;

        Ok(Json(projects))
    }
}
