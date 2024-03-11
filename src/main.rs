mod database;
mod controllers;

use std::{env, sync::Arc};
use axum::{
    routing::{get, post}, Router
};
use database::PostgresRepo;

pub type AppState = Arc<PostgresRepo>;

#[tokio::main]
async fn main() {
    let port = env::var("PORT")
        .ok()
        .and_then(|port| port.parse::<u16>().ok())
        .unwrap_or(9999);

    let url = env::var("DATABASE_URL")
        .unwrap_or(String::from("postgres://postgres:postgres@localhost:5432/postgres"));
    
    let conn = PostgresRepo::connect(url).await;

    let app_state = Arc::new(conn);

    let app = Router::new()
        .route("/pessoas", get(controllers::search_person))
        .route("/pessoas/:id", get(controllers::search_by_id))
        .route("/pessoas", post(controllers::create_person))
        .route("/contagem-pessoas", get(controllers::count_person))
        .with_state(app_state);

    let url_server = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(url_server).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
