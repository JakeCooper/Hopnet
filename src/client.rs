use std::env;
use redis::{Client, RedisError};

pub async fn create_client(redis_uri: String) -> Result<Client, RedisError> {
    Ok(Client::open(redis_uri)?)
}

pub async fn redis() -> redis::Client {
    let redis_url = match env::var("REDIS_URL") {
        Ok(v) => v,
        Err(_) => panic!("REDIS_URL must be provided!")
    };

    let redis_client = create_client(redis_url)
        .await
        .expect("Can't create Redis client");

    return redis_client;
}

