use std::{marker::PhantomData, sync::Arc};

use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, types::Json, Row, SqlitePool};

pub struct JsonCell<T> {
  pool: Arc<SqlitePool>,
  key: String,
  marker: PhantomData<T>,
}

impl<T> JsonCell<T> {
  pub fn new(pool: Arc<SqlitePool>, key: String) -> Self {
    let marker = PhantomData;
    Self { pool, key, marker }
  }
}

impl<T> JsonCell<T>
where
  for<'a> T:
    Serialize + Deserialize<'a> + Unpin + Send + Sync + 'static,
{
  pub async fn get(&self) -> sqlx::Result<Option<T>> {
    sqlx::query("SELECT * FROM objects WHERE key == ?")
      .bind(&self.key)
      .try_map(|row: SqliteRow| row.try_get("value"))
      .try_map(|value: String| {
        Json::decode_from_string(&value).map_err(sqlx::Error::Decode)
      })
      .map(|Json(value)| value)
      .fetch_optional(&*self.pool)
      .await
  }

  pub async fn save(&self, value: &T) -> sqlx::Result<()> {
    let value = Json(value);
    sqlx::query!(
      "REPLACE INTO objects (key, value) VALUES (?, ?)",
      self.key,
      value
    )
    .execute(&*self.pool)
    .await?;
    Ok(())
  }

  pub async fn remove(&self) -> sqlx::Result<()> {
    sqlx::query!("DELETE FROM objects WHERE key == ?", self.key)
      .execute(&*self.pool)
      .await?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use serde::{Deserialize, Serialize};
  use sqlx::sqlite::SqlitePoolOptions;
  use tokio::sync::Semaphore;

  use crate::{common::Result, db::Db};

  const TEST_DATABASE_URL: &str = "sqlite://db/test.db";
  static SHARED_TESTS_GUARD: Semaphore = Semaphore::const_new(1);

  async fn test_db() -> Result<Db> {
    let sqlite_pool = SqlitePoolOptions::new()
      .idle_timeout(Duration::from_millis(100))
      .acquire_timeout(Duration::from_millis(100))
      .connect(TEST_DATABASE_URL)
      .await?;
    Ok(Db::new(sqlite_pool))
  }

  #[derive(Debug, PartialEq, Serialize, Deserialize)]
  struct Data {
    value: u8,
  }

  #[tokio::test]
  async fn read_write_remove() {
    const DATA: Data = Data { value: 42 };
    let _guard = SHARED_TESTS_GUARD.acquire().await.unwrap();
    let json_cell = test_db().await.unwrap().json_cell("test_cell");
    json_cell.save(&DATA).await.unwrap();
    let data = json_cell.get().await.unwrap();
    assert_eq!(Some(DATA), data);
    json_cell.remove().await.unwrap();
    let data = json_cell.get().await.unwrap();
    assert_eq!(None, data);
  }
}
