use std::{env, error::Error, time::Duration};

use deadpool_redis::{ConnectionAddr, ConnectionInfo, PoolConfig, RedisConnectionInfo, Timeouts};
use deadpool::{managed::QueueMode::Lifo, Runtime};
use config::{Config, ConfigError, Environment};
use serde::Deserialize;

type ResultPoolError<T> = Result<T, Box<(dyn Error + Send + Sync + 'static)>>;

#[derive(Deserialize)]
pub struct DbConfig {
    pub host: String,
    pub port: u64,
    pub name: String,
    pub user: String,
    pub pwd: String,
    pub pool: u32
}

impl DbConfig {
    pub fn new() -> Result<DbConfig, ConfigError> {
        Config::builder()
            .add_source(Environment::with_prefix("DB"))
            .build()?
            .try_deserialize()
    }       
}

pub fn init_redis_pool() -> ResultPoolError<deadpool_redis::Pool> {
    let mut cfg = deadpool_redis::Config::default();
    let redis_host = env::var("REDIS_HOST").unwrap_or("0.0.0.0".into());
    
    cfg.connection = Some(ConnectionInfo {
        addr: ConnectionAddr::Tcp(redis_host, 6379),
        redis: RedisConnectionInfo {
            db: 0,
            username: None,
            password: None,
        },
    });
    
    cfg.pool = Some(PoolConfig {
        max_size: 9995,
        timeouts: Timeouts {
            wait: Some(Duration::from_secs(60)),
            create: Some(Duration::from_secs(60)),
            recycle: Some(Duration::from_secs(60)),
        },
        queue_mode: Lifo
    });
    
    println!("creating redis pool...");
    let redis_pool = cfg.create_pool(Some(Runtime::Tokio1))?;
    println!("redis pool succesfully created");

    Ok(redis_pool)
}
