mod database;

use std::{env, sync::Arc};

use axum::{
    extract::{Path, Query, State}, http::StatusCode, response::IntoResponse, routing::{get, post}, Json, Router
};
use database::PostgresRepo;
use serde::{Deserialize, Serialize};
use time::Date;

use uuid::Uuid;

time::serde::format_description!(date_format, Date, "[year]-[month]-[day]");

#[derive(Serialize, Clone, Deserialize)]
#[serde(try_from="String")]
pub struct PersonName(String);

#[derive(Serialize, Clone, Deserialize)]
#[serde(try_from="String")]
pub struct PersonNick(String);

#[derive(Serialize, Clone, Deserialize)]
#[serde(try_from="String")]
pub struct Tech(String);

impl TryFrom<String> for PersonName{
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() <= 100 {
            Ok(Self(value))
        } else {
            Err("Name is too Big!")
        }
    }
}

impl TryFrom<String> for PersonNick {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() <= 32 {
            Ok(Self(value))
        } else {
            Err("Nick is too big or Null")
        }
    }
}

impl TryFrom<String> for Tech {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() <= 32 {
            Ok(Self(value))
        } else {
            Err("Tech is too big")
        }
    }
}

impl From<Tech> for String {
    fn from(tech: Tech) -> Self {
        tech.0
    }
}

#[derive(Deserialize)]
struct PersonSearchQuery {
    #[serde(rename="t")]
    query: String
}

#[derive(Serialize, Clone, sqlx::FromRow)]
pub struct Person {
    pub id: Uuid,
    #[serde(rename = "nome")]
    pub name: String,
    #[serde(rename = "apelido")]
    pub nick: String,
    #[serde(rename = "nascimento", with = "date_format")]
    pub birth_date: Date,
    pub stack: Option<Vec<String>>
}

#[derive(Deserialize)]
pub struct NewPerson {
    #[serde(rename = "nome")]
    pub name: PersonName,
    #[serde(rename = "apelido")]
    pub nick: PersonNick,
    #[serde(rename = "nascimento", with = "date_format")]
    pub birth_date: Date,
    pub stack: Option<Vec<Tech>>
}

type AppState = Arc<PostgresRepo>;

#[tokio::main]
async fn main() {
    let url = env::var("DATABASE_URL")
        .unwrap_or(String::from("postgres://postgres:postgres@localhost:5432/postgres"));
    
    let conn = PostgresRepo::connect(url).await;

    let app_state = Arc::new(conn);

    let app = Router::new()
        .route("/pessoas", get(search_person))
        .route("/pessoas/:id", get(search_by_id))
        .route("/pessoas", post(create_person))
        .route("/contagem-pessoas", get(count_person))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9999").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn search_person(
    State(people): State<AppState>, 
    // Query(PersonSearchQuery {query}): Query<PersonSearchQuery>
    query: Query<PersonSearchQuery>
) -> impl IntoResponse {
    dbg!(&query.query);
    match people.search_person(query.query.to_string()).await {
        Ok(people) => Ok(Json(people)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

async fn search_by_id(
    State(people): State<AppState>, 
    Path(person_id): Path<Uuid>
) -> impl IntoResponse {
    match people.find_person(dbg!(person_id)).await {
        Ok(Some(person)) => Ok(Json(person)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn create_person(
    State(people): State<AppState>,
    Json(new_person): Json<NewPerson>
) -> impl IntoResponse {
    match people.create_person(new_person).await {
        Ok(person) => Ok((StatusCode::CREATED, Json(person))),
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
            Err(StatusCode::UNPROCESSABLE_ENTITY)
        },
        Err(_) => {
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn count_person(
    State(people): State<AppState>
) -> impl IntoResponse {
    match people.count_people().await {
        Ok(count) => Ok(Json(count)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}