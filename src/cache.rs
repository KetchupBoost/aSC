use std::error::Error;

use fred::{clients::RedisPool, interfaces::{ClientLike, KeysInterface, ServerInterface}, types::{Builder, Expiration, ReconnectPolicy, RedisConfig, RespVersion, SetOptions}};

use crate::{config::RedisCfg, controllers::Person, error_handler::{DBError, DatabaseError}};

#[derive(Debug, Clone)]
pub struct RedisConn {
    pub pool: RedisPool
}


impl RedisConn {
    pub async fn connect(rd_cfg: RedisCfg, flush: bool) -> DBError<Self> {
        let config = RedisConfig::from_url(&rd_cfg.url)?;

        let redis_pool = Builder::from_config(config)
            .with_performance_config(|config| {
                config.auto_pipeline = true;
            })
            .with_config(|config| {
                config.version = RespVersion::RESP3;
            })
            .set_policy(ReconnectPolicy::new_exponential(0, 100, 30_000, 2))
            .build_pool(rd_cfg.pool)?;
        redis_pool.init().await?;

        if flush { 
            let _ = redis_pool.flushall::<i32>(false).await?;
        }

        Ok(Self { pool: redis_pool })
    }

    pub async fn connected(&self) -> DBError<bool> {
        if !self.pool.is_connected() {   
            return Err(
                DatabaseError::RedisError(
                    Box::new(simple_error::SimpleError::new("Redis is not connected."))
                )
            )
        } else {
            tracing::info!("================ Redis is connected =================");
            Ok(true)
        }
    }

    pub async fn get_person(&self, nick: String) -> DBError<Option<Person>> {
        let value: Option<_> = self.pool.get(nick).await?;

        let person = match value {
            Some(x) => match serde_json::from_value(x) {
                Ok(x) => Some(x),
                Err(_) => None,
            },
            None => None,
        };
        Ok(person)
    }

    pub async fn set_person(
        &self,
        nick: String,
        person: Person,
        expiration: Option<Expiration>,
        set_opts: Option<SetOptions>,
        get: bool
    ) -> Result<(), Box<dyn Error>> {
        let value: _ = serde_json::to_value(person)?.to_string();
        Ok(
            self.pool.set(
                nick, 
                value,
                expiration, 
                set_opts, 
                get
            ).await?
        )
    }
}