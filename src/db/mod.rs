pub mod json_cell;
pub mod txn;

use std::{env, error::Error, sync::Arc};

use sqlx::{sqlite::SqlitePoolOptions, Result, SqlitePool};

pub use json_cell::JsonCell;

use self::txn::ExecutorHolder;

const DATABASE_URL: &str = "DATABASE_URL";

pub async fn connect() -> Result<Db, Box<dyn Error>> {
  let url = env::var(DATABASE_URL)?;
  let pool = SqlitePoolOptions::new().connect(&url).await?;
  Ok(Db::new(pool))
}

pub struct Db {
  pool: Arc<SqlitePool>,
}

impl Db {
  fn new(pool: SqlitePool) -> Self {
    Self { pool: pool.into() }
  }

  pub fn pool(&self) -> Arc<SqlitePool> {
    self.pool.clone()
  }

  #[allow(unused)]
  pub fn json_cell<T>(&self, key: impl Into<String>) -> JsonCell<T> {
    JsonCell::new(self.pool.clone(), key.into())
  }

  pub fn exec(&self) -> ExecutorHolder {
    ExecutorHolder::new(self.pool())
  }
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use sqlx::sqlite::SqlitePoolOptions;

  use crate::common::Result;

  use super::Db;

  const TEST_DATABASE_URL: &str = "sqlite://db/test.db";

  pub async fn test_db() -> Result<Db> {
    let sqlite_pool = SqlitePoolOptions::new()
      .idle_timeout(Duration::from_millis(100))
      .acquire_timeout(Duration::from_millis(100))
      .connect(TEST_DATABASE_URL)
      .await?;
    Ok(Db::new(sqlite_pool))
  }
}
