use axum::{
    routing::{get, post, IntoMakeService},
    Extension,
};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

use super::endpoint;

pub struct Router {
    axum_router: axum::Router,
}

pub struct JwtExt {
    pub secret: String,
}

impl Router {
    pub fn new(pool: Pool<Postgres>, jwt_secret: &str) -> Self {
        let jwt_ext = Arc::new(JwtExt {
            secret: jwt_secret.to_owned(),
        });

        let router = axum::Router::new()
            .route("/users", post(endpoint::user::create_user))
            .route("/users/login", post(endpoint::user::login))
            .route("/user", get(endpoint::user::get_user))
            .with_state(pool)
            .layer(TraceLayer::new_for_http())
            .layer(Extension(jwt_ext));

        Self {
            axum_router: router,
        }
    }

    pub fn into_make_service(self) -> IntoMakeService<axum::Router> {
        self.axum_router.into_make_service()
    }
}
