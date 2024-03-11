use std::vec;

use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use uuid::Uuid;

use crate::controllers::{NewPerson, Person, PersonName, PersonNick};

#[derive(Debug)]
pub struct PostgresRepo {
    pub pool: PgPool,
}

impl PostgresRepo {
    pub async fn connect(url: String) -> Self {
        PostgresRepo {
            pool: PgPoolOptions::new()
                .max_connections(5)
                .connect(&url)
                .await
                .unwrap()
        }
    }

    pub async fn find_person(&self, id: Uuid) -> Result<Option<Person>, sqlx::Error> {
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
    }

    pub async fn create_person(&self, new_person: NewPerson) -> Result<Person, sqlx::Error> {
        let stack = match &new_person.stack {
            Some(stack) => {
                stack
                    .into_iter()
                    .map(|tech| String::from(tech.clone()))
                    .collect::<Vec<_>>()
            },
            None => vec![]
        };

        sqlx::query_as!(
            Person,
            "
            INSERT INTO person(id, name, nick, birth_date, stack)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, nick, birth_date, stack
            ",
            Uuid::now_v7(),
            PersonName::get_name(&new_person),
            PersonNick::get_nick(&new_person),
            new_person.birth_date,
            &stack  
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn search_person(&self, query: String) -> Result<Vec<Person>, sqlx::Error> {
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
    }

    pub async fn count_people(&self) -> Result<i64, sqlx::Error> {
        sqlx::query("SELECT count(*) FROM person")
            .fetch_one(&self.pool)
            .await
            .map(|row| row.get(0))
    }
}