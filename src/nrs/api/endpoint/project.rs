pub(crate) mod router {
    use axum::routing::{self, get};
    use sqlx::{Pool, Postgres};

    use super::handler;

    pub fn new(pool: &Pool<Postgres>) -> routing::Router {
        routing::Router::new()
            .route("/", get(handler::get))
            .with_state(pool.clone())
    }
}

mod handler {
    use axum::{extract::State, Json};
    use sqlx::PgPool;

    use crate::api::extract::AuthUser;
    use crate::api::Result;

    mod request {}

    mod response {
        use serde::Serialize;

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
