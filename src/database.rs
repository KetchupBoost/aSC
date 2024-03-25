
use std::sync::Arc;

use axum::{extract::State, routing::get};
use fred::types::{Expiration, SetOptions};
use sqlx::Row;
use uuid::Uuid;
use sqlx::{postgres::PgPoolOptions, PgPool};
use crate::{cache::RedisConn, config::DbConfig, controllers::{AppState, NewPerson, Person, PersonName, PersonNick}, error_handler::{DBError, DatabaseError}};

#[derive(Debug)]
pub struct PostgresConn {
    pub pool: PgPool,
}

impl PostgresConn {
    pub async fn connect(cfg: DbConfig) -> DBError<Self> {
        let pg = PostgresConn {
                pool: PgPoolOptions::new()
                    .max_connections(cfg.pool)
                    .connect(&cfg.url)
                    .await?
        };
        
        sqlx::migrate!().run(&pg.pool).await?;

        Ok(pg)
    }

    pub async fn find_person(&self, id: Uuid) -> DBError<Option<Person>> {
        sqlx::query_as!(
            Person,
            "
            SELECT id, name, nick, birth_date, stack
            FROM person
            WHERE id=$1
            ",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::from)
    }

    pub async fn create_person(&self, new_person: NewPerson, state: &AppState) -> DBError<Person> {
        let person_name = PersonName::get_name(&new_person);
        let person_nick = PersonNick::get_nick(&new_person);

        let _ = state.cache.connected().await?;

        let get_person = state.cache.get_person(person_nick.clone()).await?; 
        match get_person { 
            Some(_) => return Err(DatabaseError::UniqueViolation),
            None => ()
        };

        let stack = match &new_person.stack {
            Some(stack) => {
                stack
                    .into_iter()
                    .map(|tech| String::from(tech.clone()))
                    .collect::<Vec<_>>()
            },
            None => vec![]
        };

        let person = sqlx::query_as!(
            Person,
            "
            INSERT INTO person(id, name, nick, birth_date, stack)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, nick, birth_date, stack
            ",
            Uuid::now_v7(),
            &person_name,
            &person_nick,
            &new_person.birth_date,
            &stack
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DatabaseError::from)?;

        {
            let state = state.clone();
            let person = person.clone();

            tokio::spawn(async move {
                let _ = state.cache.set_person(
                    person_nick, 
                    person, 
                    Some(Expiration::KEEPTTL), 
                    Some(SetOptions::NX), 
                    true
                ).await.unwrap();
            });
        }
        Ok(person)
    }

    pub async fn search_person(&self, query: String) -> DBError<Vec<Person>> {
        sqlx::query_as!(
            Person,
            "
            SELECT id, name, nick, birth_date, stack
            FROM person
            WHERE to_tsquery('person', $1) @@ search
            LIMIT 50
            ",
            query
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::from)
    }

    pub async fn count_people(&self) -> DBError<i64> {
        sqlx::query("SELECT COUNT(*) AS count FROM person")
            .fetch_one(&self.pool)
            .await
            .map(|row| row.get(0))
            .map_err(DatabaseError::from)
    }
}