use axum::{
    middleware,
    routing::{delete, get, post, IntoMakeService},
    Extension,
};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

use super::{endpoint, middleware::console::log_body};

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

        use endpoint::user;

        let router = axum::Router::new()
            .route("/users", post(user::create))
            .route("/users/login", post(user::login))
            .route("/user", get(user::get))
            .route("/user", delete(user::delete))
            .with_state(pool)
            .layer(TraceLayer::new_for_http())
            .layer(Extension(jwt_ext))
            .layer(middleware::from_fn(log_body));

        Self {
            axum_router: router,
        }
    }

    pub fn into_make_service(self) -> IntoMakeService<axum::Router> {
        self.axum_router.into_make_service()
    }
}
