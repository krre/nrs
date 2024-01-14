pub(crate) mod router {
    use axum::routing::{self, delete, get, post, put};
    use sqlx::{Pool, Postgres};

    use super::handler;

    pub fn new(pool: &Pool<Postgres>) -> routing::Router<Pool<Postgres>> {
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

    use crate::api::extract::ValidPayload;
    use crate::api::Result;

    mod request {
        use serde::Deserialize;
        use validator::Validate;

        #[derive(Deserialize, Validate)]
        pub struct Create {
            pub project_id: i64,
            #[validate(length(min = 1))]
            pub name: String,
            pub visibility: i16,
        }

        #[derive(Deserialize, Validate)]
        pub struct Update {
            #[validate(length(min = 1))]
            pub name: String,
            pub visibility: i16,
        }
    }

    mod response {
        use serde::Serialize;

        #[derive(Serialize)]
        pub struct Create {
            pub id: i64,
        }

        #[derive(Serialize)]
        pub struct Module {
            pub id: i64,
            pub project_id: i64,
            pub name: String,
            pub visibility: i16,
            pub updated_at: chrono::DateTime<chrono::Local>,
        }
    }

    pub async fn create(
        Path(project_id): Path<i64>,
        State(pool): State<PgPool>,
        ValidPayload(payload): ValidPayload<request::Create>,
    ) -> Result<Json<response::Create>> {
        struct Module {
            id: i64,
        }

        let module = sqlx::query_as!(
            Module,
            "INSERT INTO modules (project_id, name, visibility) values ($1, $2, $3) RETURNING id",
            project_id,
            payload.name,
            payload.visibility,
        )
        .fetch_one(&pool)
        .await?;

        Ok(Json(response::Create { id: module.id }))
    }

    pub async fn update(
        Path(id): Path<i64>,
        State(pool): State<PgPool>,
        ValidPayload(payload): ValidPayload<request::Update>,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE modules SET name = $1, visibility = $2, updated_at = current_timestamp WHERE id = $3",
            payload.name,
            payload.visibility,
            id,
        )
        .execute(&pool)
        .await?;

        Ok(())
    }

    pub async fn get_all(
        Path(project_id): Path<i64>,
        State(pool): State<PgPool>,
    ) -> Result<Json<Vec<response::Module>>> {
        let projects = sqlx::query_as!(
            response::Module,
            "SELECT id, project_id, name, visibility, updated_at FROM modules
            WHERE project_id = $1
            ORDER BY updated_at DESC",
            project_id,
        )
        .fetch_all(&pool)
        .await?;

        Ok(Json(projects))
    }

    pub async fn get_one(
        Path(id): Path<i64>,
        State(pool): State<PgPool>,
    ) -> Result<Json<response::Module>> {
        let project = sqlx::query_as!(
            response::Module,
            "SELECT id, project_id, name, visibility, updated_at FROM modules
            WHERE id = $1",
            id,
        )
        .fetch_one(&pool)
        .await?;

        Ok(Json(project))
    }

    pub async fn delete(Path(id): Path<i64>, State(pool): State<PgPool>) -> Result<()> {
        sqlx::query!("DELETE FROM modules WHERE id = $1", id)
            .execute(&pool)
            .await?;

        Ok(())
    }
}
