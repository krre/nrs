use axum::routing::{post, IntoMakeService};
use sqlx::{Pool, Postgres};

use crate::api;

pub struct Router {
    axum_router: axum::Router,
}

impl Router {
    pub fn new(pool: Pool<Postgres>) -> Self {
        let router = axum::Router::new()
            .route("/users", post(api::user::create_user))
            .with_state(pool);

        Self {
            axum_router: router,
        }
    }

    pub fn into_make_service(self) -> IntoMakeService<axum::Router> {
        self.axum_router.into_make_service()
    }
}
