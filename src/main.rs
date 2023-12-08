use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use shuttle_runtime::CustomError;
use sqlx::{FromRow, PgPool};

async fn retrieve(
    Path(id): Path<i32>,
    State(state): State<MyState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query_as::<_, Quote>("SELECT * FROM quotes WHERE id = $1")
        .bind(id)
        .fetch_one(&state.pool)
        .await
    {
        Ok(quote) => Ok((StatusCode::OK, Json(quote))),
        Err(_e) => Err((StatusCode::NOT_FOUND, "Not Found")),
    }
}

async fn random(State(state): State<MyState>) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query_as::<_, Quote>("SELECT * FROM quotes ORDER BY RANDOM() LIMIT 1")
        .fetch_one(&state.pool)
        .await
    {
        Ok(quote) => Ok((StatusCode::OK, Json(quote))),
        Err(_e) => Err((StatusCode::NOT_FOUND, "Not Found")),
    }
}

async fn add(
    State(state): State<MyState>,
    Json(data): Json<QuoteNew>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query_as::<_, Quote>(
        "INSERT INTO quotes (quote, speaker, source) VALUES ($1, $2, $3) RETURNING id, quote, speaker, source",
    )
    .bind(&data.quote)
    .bind(&data.speaker)
    .bind(&data.source)
    .fetch_one(&state.pool)
    .await
    {
        Ok(todo) => Ok((StatusCode::CREATED, Json(todo))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

#[derive(Clone)]
struct MyState {
    pool: PgPool,
}

#[shuttle_runtime::main]
async fn axum(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(CustomError::new)?;

    let state = MyState { pool };
    let router = Router::new()
        .route("/", get(random))
        // .route("/hello", get(|| async { "Hello, World!" }))
        // .route("/quotes", post(add))
        .route("/quotes/:id", get(retrieve))
        .with_state(state);

    Ok(router.into())
}

#[derive(Deserialize)]
struct QuoteNew {
    pub quote: String,
    pub speaker: String,
    pub source: String,
}

#[derive(Serialize, FromRow)]
struct Quote {
    pub id: i32,
    pub quote: String,
    pub speaker: String,
    pub source: String,
}
