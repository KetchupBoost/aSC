mod database;
mod controllers;
mod config;
mod cache;
mod error_handler;

use std::{env, error::Error, sync::Arc};
use axum::{
    routing::{get, post}, Router
};
use cache::RedisConn;
use database::PostgresConn;
use dotenv::dotenv;

pub type AsyncVoidResult = Result<(), Box<dyn Error + Send + Sync>>;

#[derive(Debug)]
pub struct InternalState {
    database: PostgresConn,
    cache: RedisConn 
}

#[tokio::main]
async fn main() -> AsyncVoidResult {
    dotenv().ok();

    // initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let db_cfg = config::DbConfig::get()?;
    let db_conn = PostgresConn::connect(db_cfg).await?;

    let rd_cfg = config::RedisCfg::get()?;
    let rd_conn = RedisConn::connect(rd_cfg, false).await?;

    let int_state = InternalState {
        database: db_conn,
        cache: rd_conn
    };

    let app_state = Arc::new(int_state);

    let app = Router::new()
        .route("/pessoas", get(controllers::search_person))
        .route("/pessoas/:id", get(controllers::search_by_id))
        .route("/pessoas", post(controllers::create_person))
        .route("/contagem-pessoas", get(controllers::count_person))
        .with_state(app_state);

    let port = env::var("PORT")
        .ok()
        .and_then(|port| port.parse::<u16>().ok())
        .unwrap_or(9999);

    let url_server = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(url_server).await?;
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
