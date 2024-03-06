use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, State}, http::StatusCode, response::IntoResponse, routing::{get, post}, Error, Json, Router
};
use serde::{Deserialize, Serialize};
use time::Date;
use tokio::sync::RwLock;
use uuid::Uuid;

time::serde::format_description!(date_format, Date, "[year]-[month]-[day]");

#[derive(Serialize, Clone, Deserialize)]
#[serde(try_from="String")]
pub struct PersonName(String);

impl TryFrom<String> for PersonName{
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() <= 100 {
            Ok(PersonName(value))
        } else {
            Err("Name is too Big!")
        }
    }
}

#[derive(Serialize, Clone)]
pub struct Person {
    pub id: Uuid,
    #[serde(rename = "nome")]
    pub name: PersonName,
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
    pub nick: String,
    #[serde(rename = "nascimento", with = "date_format")]
    pub birth_date: Date,
    pub stack: Option<Vec<String>>
}

type AppState = Arc<RwLock<HashMap<Uuid, Person>>>;

#[tokio::main]
async fn main() {
    let people: HashMap<Uuid, Person> = HashMap::new(); 
    let app_state = Arc::new(RwLock::new(people));

    let app = Router::new()
        .route("/pessoas", get(search_all))
        .route("/pessoas/:id", get(search_by_id))
        .route("/pessoas", post(create_person))
        .route("/contagem-pessoas", get(count_person))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9999").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn search_all() -> impl IntoResponse {
    StatusCode::OK
}

async fn search_by_id(
    State(people): State<AppState>, 
    Path(person_id): Path<Uuid>
) -> impl IntoResponse {
    match people.read().await.get(&dbg!(person_id)){
        Some(person) => Ok(Json(person.clone())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn create_person(
    State(people): State<AppState>,
    Json(new_person): Json<NewPerson>
) -> impl IntoResponse {
    if new_person.name.len() > 100
    || new_person.nick.len() > 32
    {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    
    if let Some(ref stack) =  new_person.stack {
        if stack.iter().any(|value| value.len() > 32 ) {
            return Err(StatusCode::UNPROCESSABLE_ENTITY)
        }
    }

    let uuid = Uuid::now_v7();
    let person = Person {
        id: uuid,
        name: new_person.name,
        nick: new_person.nick,
        birth_date: new_person.birth_date,
        stack: new_person.stack
    };

    people.write().await.insert(uuid, person.clone());
    Ok((StatusCode::OK, Json(person)))
}

async fn count_person() -> impl IntoResponse {
    StatusCode::OK
}