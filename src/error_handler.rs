use std::{error::Error, fmt::{Display, Formatter}};

pub type DBError<T> = Result<T, DatabaseError>;
pub type RedisError = fred::error::RedisError;

#[derive(Debug)]
pub enum DatabaseError {
    UniqueViolation,
    DatabaseError(Box<dyn Error + Send + Sync>),
    MigrateError(Box<dyn Error + Send + Sync>),
    RedisError(Box<dyn Error + Send + Sync>),
}

// Treating Errors 
impl Error for DatabaseError {}

impl Display for DatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UniqueViolation => write!(f, "Error - Unique Violation!"),
            Self::DatabaseError(e) => write!(f, "Error in Database - {e}"),
            Self::MigrateError(e) => write!(f, "Error Migrating DataBase - {e}"),
            Self::RedisError(e) => write!(f, "Error in to Redis - {e}")
        }
    }
}

impl From<sqlx::Error> for DatabaseError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::Database(err) if err.is_unique_violation() => {
                DatabaseError::UniqueViolation
            },
            _ => DatabaseError::DatabaseError(Box::new(error)),
        }
    }
}

impl From<sqlx::migrate::MigrateError> for DatabaseError {
    fn from(value: sqlx::migrate::MigrateError) -> Self {
        DatabaseError::MigrateError(Box::new(value))
    }
}

impl From<RedisError> for DatabaseError {
    fn from(value: RedisError) -> Self {
        DatabaseError::RedisError(Box::new(value))
    }
}
