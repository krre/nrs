use axum::extract;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateUser {
    name: String,
    email: String,
    password: String,
}

pub async fn create_user(extract::Json(payload): extract::Json<CreateUser>) {
    println!("{} {} {}", payload.name, payload.email, payload.password);
}
