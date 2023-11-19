use axum::{async_trait, body::HttpBody, extract::FromRequest, http::Request, BoxError, Json};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::api;

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidPayload<T>(pub T);

#[async_trait]
impl<T, S, B> FromRequest<S, B> for ValidPayload<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    B: Send + HttpBody + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = api::Error;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidPayload(value))
    }
}
