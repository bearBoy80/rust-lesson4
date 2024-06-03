use std::sync::Arc;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool};
use thiserror::Error;
pub mod models;

pub const ADDR: &str = "127.0.0.1:8080";

#[derive(Debug, Clone)]
pub struct AppState {
    pub inner: Arc<AppStateInner>,
}
#[allow(unused)]
#[derive(Debug)]
pub struct AppStateInner {
    pub pool: PgPool,
}
impl AppState {
    pub async fn try_new() -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect("postgres://postgres:postgres@localhost:5432/shorturl")
            .await?;
        let app_state_inner = AppStateInner { pool };
        Ok(Self {
            inner: Arc::new(app_state_inner),
        })
    }
}
#[derive(Error, Debug)]
pub enum AppError {
    #[error("pgdb slq error info : {0}")]
    PgDbErr(#[from] sqlx::Error),
    #[error("duplicate key value violates unique constraint : {0}")]
    PgDuplicate(String),
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::PgDbErr(ref err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", err),
            ),
            AppError::PgDuplicate(ref err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("duplicate key error: {}", err),
            ),
        };

        let body = Json(json!(
           { "error_message":error_message,}
        ));
        (status, body).into_response()
    }
}

#[derive(Debug, Deserialize)]
pub struct ShortenReq {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct ShortenRes {
    pub url: String,
}
