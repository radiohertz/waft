use axum::{Router, routing::get};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Hellow {
    msg: String,
}

pub async fn index() -> axum::response::Json<Hellow> {
    Hellow { msg: "hi".into() }.into()
}

pub fn router() -> axum::Router {
    Router::new().route_service("/", get(index))
}
