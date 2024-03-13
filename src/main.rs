mod database;
mod controllers;
mod config;

use std::{env, error::Error, sync::Arc};
use axum::{
    routing::{get, post}, Router
};
use database::PostgresRepo;
use dotenv::dotenv;

pub type AppState = Arc<PostgresRepo>;
pub type AsyncVoidResult = Result<(), Box<dyn Error + Send + Sync>>;

#[tokio::main]
async fn main() -> AsyncVoidResult {
    dotenv().ok();

    let cfg = config::DbConfig::new()?;
    let conn = PostgresRepo::connect(cfg).await.unwrap();

    let _ = config::init_redis_pool()?;

    let port = env::var("PORT")
        .ok()
        .and_then(|port| port.parse::<u16>().ok())
        .unwrap_or(9999);

    let app_state = Arc::new(conn);

    let app = Router::new()
        .route("/pessoas", get(controllers::search_person))
        .route("/pessoas/:id", get(controllers::search_by_id))
        .route("/pessoas", post(controllers::create_person))
        .route("/contagem-pessoas", get(controllers::count_person))
        .with_state(app_state);

    let url_server = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(url_server).await?;
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
