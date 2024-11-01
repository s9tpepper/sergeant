use std::env;

use sqlx::{Pool, Sqlite, SqlitePool};

pub async fn get_connection_pool() -> anyhow::Result<Pool<Sqlite>> {
    Ok(SqlitePool::connect(&env::var("DATABASE_URL")?).await?)
}
