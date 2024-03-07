use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use uuid::Uuid;

use crate::{NewPerson, Person};

#[derive(Debug)]
pub struct PostgresRepo {
    pool: PgPool,
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
        sqlx::query_as(
            "
            SELECT id, name, birth_date, stack
            FROM people
            WHERE id=$1
            ",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn create_person(&self, new_person: NewPerson) -> Result<Person, sqlx::Error> {
        sqlx::query_as(
            "
            INSERT INTO person(id, name, nick, birth_date, stack)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, nick, birth_date, stack
            "
        )
        .bind(Uuid::now_v7())
        .bind(new_person.name.0)
        .bind(new_person.nick.0)
        .bind(new_person.birth_date)
        .bind(
            new_person
                .stack
                .map(|stk| stk.into_iter().map(String::from).collect::<Vec<String>>())
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn search_person(&self, query: String) -> Result<Vec<Person>, sqlx::Error> {
        dbg!(&query);
        sqlx::query_as(
            "
            SELECT id, name, nick, birth_date, stack
            FROM person
            WHERE to_tsquery('person', $1) @@ search
            LIMIT 50
            "
        )
        .bind(query)
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