use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
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
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/json; charset=utf-8".parse().unwrap(),
    );

    match sqlx::query_as::<_, Quote>("SELECT quote, speaker, source FROM quotes WHERE id = $1")
        .bind(id)
        .fetch_one(&state.pool)
        .await
    {
        Ok(quote) => Ok((StatusCode::OK, headers, Json(quote))),
        Err(_e) => Err((StatusCode::NOT_FOUND, "Not Found")),
    }
}

async fn random(State(state): State<MyState>) -> Result<impl IntoResponse, impl IntoResponse> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/json; charset=utf-8".parse().unwrap(),
    );

    match sqlx::query_as::<_, Quote>(
        "SELECT quote, speaker, source FROM quotes ORDER BY RANDOM() LIMIT 1",
    )
    .fetch_one(&state.pool)
    .await
    {
        Ok(quote) => Ok((StatusCode::OK, headers, Json(quote))),
        Err(_e) => Err((StatusCode::NOT_FOUND, "Not Found")),
    }
}

async fn retrieve_source(
    Path(id): Path<i32>,
    State(state): State<MyState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/json; charset=utf-8".parse().unwrap(),
    );

    let source = format!("第{}部", id);

    match sqlx::query_as::<_, Quote>("SELECT quote, speaker, source FROM quotes WHERE source = $1 ORDER BY RANDOM() LIMIT 1")
        .bind(source)
        .fetch_one(&state.pool)
        .await
    {
        Ok(quote) => Ok((StatusCode::OK, headers, Json(quote))),
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
async fn axum(#[shuttle_shared_db::Postgres(local_uri = "postgres://postgres:postgres@localhost:15660")] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(CustomError::new)?;

    let state = MyState { pool };
    let router = Router::new()
        .route("/", get(random))
        .route("/:id", get(retrieve_source))
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
    pub quote: String,
    pub speaker: String,
    pub source: String,
}
