use std::{error::Error, vec};
use sqlx::Row;
use uuid::Uuid;
use sqlx::{postgres::PgPoolOptions, PgPool};
use crate::{config::DbConfig, controllers::{NewPerson, Person, PersonName, PersonNick}};

pub enum DatabaseError {
    UniqueViolation,
    DatabaseError(Box<dyn Error + Send + Sync>),
}

impl From<sqlx::Error> for DatabaseError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::Database(err) if err.is_unique_violation() => {
                DatabaseError::UniqueViolation
            }
            _ => DatabaseError::DatabaseError(Box::new(error)),
        }
    }
}

type DBError<T> = Result<T, DatabaseError>;

#[derive(Debug)]
pub struct PostgresRepo {
    pub pool: PgPool,
}

impl PostgresRepo {
    pub async fn connect(cfg: DbConfig) -> DBError<Self> {
        let url = format!(
            "postgres://{}:{}@{}:{}/{}",
            cfg.user, cfg.pwd, cfg.host, cfg.port, cfg.name
        ).to_string();

        Ok(
            PostgresRepo {
                pool: PgPoolOptions::new()
                    .max_connections(cfg.pool)
                    .connect(&url)
                    .await?
            }
        )
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

    pub async fn create_person(&self, new_person: NewPerson) -> DBError<Person> {
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
        .map_err(DatabaseError::from)
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