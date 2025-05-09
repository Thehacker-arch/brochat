use sqlx::{PgPool, Error};
use sqlx::postgres::PgPoolOptions;
use std::env;
use tokio::sync::OnceCell;

static DB_POOL: OnceCell<PgPool> = OnceCell::const_new();

pub async fn init() -> Result<(), Error> {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://user:password@localhost/db_name".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(10)  
        .min_connections(1)
        .connect(&database_url).await?;

    DB_POOL.set(pool).map_err(|_| Error::Protocol("Failed to set DB pool".into()))?;

    Ok(())
}

pub async fn get_pool() -> &'static PgPool {
    DB_POOL.get().expect("Database pool is not initialized!")
}
