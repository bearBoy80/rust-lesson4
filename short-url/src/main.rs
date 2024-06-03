use axum::{
    extract::{Path, State},
    http::{header::LOCATION, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use short_url::{
    models::{get_short_url, insert_short_url},
    AppError, AppState, ShortenReq, ShortenRes, ADDR,
};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let layer = fmt::layer().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    let listener = tokio::net::TcpListener::bind(ADDR).await.unwrap();
    info!("server started:{}", ADDR);
    let app_state = AppState::try_new().await?;
    info!("Connected to database successfully");
    let app = create_router(app_state).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
async fn create_router(state: AppState) -> anyhow::Result<Router> {
    let app = Router::new()
        .route("/", post(shorten))
        .route("/:id", get(redirect))
        .with_state(state);
    Ok(app)
}
async fn shorten(
    State(state): State<AppState>,
    Json(data): Json<ShortenReq>,
) -> Result<impl IntoResponse, StatusCode> {
    let inner = state.inner;
    let url_record = insert_short_url(&inner.pool, &data.url).await?;
    let result = Json(ShortenRes {
        url: url_record.url,
    });
    Ok((StatusCode::CREATED, result).into_response())
}

async fn redirect(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let inner = state.inner;
    let url_record = get_short_url(&inner.pool, &id)
        .await
        .map_err(|_e| StatusCode::NOT_FOUND)?;
    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, url_record.parse().unwrap());
    Ok((StatusCode::PERMANENT_REDIRECT, headers))
}
