use std::sync::Arc;

use axum::{
    async_trait,
    extract::FromRequestParts,
    headers::{authorization::Bearer, Authorization},
    http::request::Parts,
    Extension, RequestPartsExt, TypedHeader,
};

use crate::{
    api,
    core::{jwt, router::JwtExt},
};

pub struct AuthUser(pub i64);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = api::error::Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| api::error::Error::BadRequest("invalid token".to_string()))?;

        let jwt_ext: Extension<Arc<JwtExt>> =
            Extension::from_request_parts(parts, state).await.unwrap();

        let user_id = jwt::user_id(bearer.token(), &jwt_ext.secret)
            .map_err(|_| api::error::Error::BadRequest("invalid token".to_string()))?;

        Ok(AuthUser(user_id))
    }
}
