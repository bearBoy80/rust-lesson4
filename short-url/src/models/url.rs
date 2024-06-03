use std::{borrow::Borrow, cell::RefCell, sync::Arc};

use nanoid::nanoid;
use sqlx::{error::DatabaseError, postgres::PgDatabaseError, Database, Error, FromRow, PgPool};
use tracing::info;

use crate::{AppError, ADDR};
#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct Url {
    pub id: String,
    pub url: String,
    pub origin_url: String,
}
pub async fn insert_short_url(db: &PgPool, origin_url: &str) -> anyhow::Result<Url, AppError> {
    //let id = nanoid!(6);
    let id = "WOo2AJ";
    let url = format!("http://{}/{}", ADDR, id);
    info!("insert_short_url- id:{:?} url :{:?}", id, url);
    let ret = sqlx::query_as(
        "INSERT INTO urls (id, url,origin_url) VALUES ($1, $2,$3) 
        ON CONFLICT(origin_url) DO UPDATE SET url=EXCLUDED.url RETURNING id,url,origin_url",
    )
    .bind(&id)
    .bind(&url)
    .bind(&origin_url)
    .fetch_one(db)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(dbe) if dbe.constraint() == Some("urls_pkey") => {
            AppError::PgDuplicate("urls_pkey".to_string())
        }
        _ => e.into(),
    })?;
    Ok(ret)
}
pub async fn get_short_url(db: &PgPool, id: &str) -> Result<String, AppError> {
    info!("get_short_url parameter :{:?}", id);
    let ret: Url = sqlx::query_as("SELECT id,origin_url,url FROM urls WHERE id = $1")
        .bind(&id)
        .fetch_one(db)
        .await?;
    info!("get_short_url - result :{:?}", ret);
    Ok(ret.origin_url)
}
