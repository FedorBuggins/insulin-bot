use std::{
  future::Future,
  sync::{Arc, Mutex},
};

use futures_core::{future::BoxFuture, stream::BoxStream};
use sqlx::{Acquire, Sqlite, SqlitePool, Transaction};

use crate::common;

tokio::task_local! {
  static SHARED_TXN: Mutex<Option<Transaction<'static, Sqlite>>>;
}

pub async fn begin<P, F>(pool: P, f: F) -> sqlx::Result<F::Output>
where
  P: AsRef<SqlitePool>,
  F: Future,
{
  let txn = pool.as_ref().begin().await?;
  Ok(SHARED_TXN.scope(Mutex::new(Some(txn)), f).await)
}

pub async fn commit() -> common::Result<()> {
  SHARED_TXN
    .try_with(|txn| txn.lock().unwrap().take())
    .map_err(|err| format!("No shared txn: {err}"))?
    .ok_or("Shared txn borrowed")?
    .commit()
    .await?;
  Ok(())
}

pub struct ExecutorHolder {
  pool: Arc<SqlitePool>,
}

impl ExecutorHolder {
  pub fn new(pool: Arc<SqlitePool>) -> Self {
    Self { pool }
  }

  /// Borrows access to shared [`Transaction`] if provided
  ///
  /// # Panics
  ///
  /// Panics if shared transaction already borrowed
  pub fn borrow(&self) -> Executor {
    match try_borrow_shared_txn() {
      Ok(Some(txn)) => Executor::Txn(Some(txn)),
      Ok(None) => panic!("Shared txn already borrowed"),
      Err(_) => Executor::Pool(self.pool.clone()),
    }
  }
}

fn try_borrow_shared_txn(
) -> common::Result<Option<Transaction<'static, Sqlite>>> {
  SHARED_TXN
    .try_with(|txn| txn.lock().unwrap().take())
    .map_err(|_| "AccessError".into())
}

#[derive(Debug)]
pub enum Executor {
  Pool(Arc<SqlitePool>),
  Txn(Option<Transaction<'static, Sqlite>>),
}

impl Executor {
  pub async fn begin(
    &mut self,
  ) -> sqlx::Result<Transaction<'_, Sqlite>> {
    match self {
      Executor::Pool(pool) => pool.begin().await,
      Executor::Txn(txn) => txn.as_mut().unwrap().begin().await,
    }
  }
}

impl Drop for Executor {
  fn drop(&mut self) {
    if let Executor::Txn(txn) = self {
      retrieve_shared_txn(txn.take().unwrap());
    }
  }
}

fn retrieve_shared_txn(txn: Transaction<'static, Sqlite>) {
  SHARED_TXN.with(|shared| *shared.lock().unwrap() = Some(txn));
}

impl<'c> sqlx::Executor<'c> for &'c mut Executor {
  type Database = Sqlite;

  fn fetch_many<'e, 'q: 'e, E: 'q>(
    self,
    query: E,
  ) -> BoxStream<
    'e,
    Result<
      sqlx::Either<
        <Self::Database as sqlx::Database>::QueryResult,
        <Self::Database as sqlx::Database>::Row,
      >,
      sqlx::Error,
    >,
  >
  where
    'c: 'e,
    E: sqlx::Execute<'q, Self::Database>,
  {
    match self {
      Executor::Pool(pool) => pool.fetch_many(query),
      Executor::Txn(txn) => txn.as_mut().unwrap().fetch_many(query),
    }
  }

  fn fetch_optional<'e, 'q: 'e, E: 'q>(
    self,
    query: E,
  ) -> BoxFuture<
    'e,
    Result<
      Option<<Self::Database as sqlx::Database>::Row>,
      sqlx::Error,
    >,
  >
  where
    'c: 'e,
    E: sqlx::Execute<'q, Self::Database>,
  {
    match self {
      Executor::Pool(pool) => pool.fetch_optional(query),
      Executor::Txn(txn) => {
        txn.as_mut().unwrap().fetch_optional(query)
      }
    }
  }

  fn prepare_with<'e, 'q: 'e>(
    self,
    sql: &'q str,
    parameters: &'e [<Self::Database as sqlx::Database>::TypeInfo],
  ) -> BoxFuture<
    'e,
    Result<
      <Self::Database as sqlx::database::HasStatement<'q>>::Statement,
      sqlx::Error,
    >,
  >
  where
    'c: 'e,
  {
    match self {
      Executor::Pool(pool) => pool.prepare_with(sql, parameters),
      Executor::Txn(txn) => {
        txn.as_mut().unwrap().prepare_with(sql, parameters)
      }
    }
  }

  fn describe<'e, 'q: 'e>(
    self,
    sql: &'q str,
  ) -> BoxFuture<
    'e,
    Result<sqlx::Describe<Self::Database>, sqlx::Error>,
  >
  where
    'c: 'e,
  {
    match self {
      Executor::Pool(pool) => pool.describe(sql),
      Executor::Txn(txn) => txn.as_mut().unwrap().describe(sql),
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::db::tests::test_db;

  use super::*;

  #[tokio::test]
  async fn should_execute_query_without_shared_txn() {
    let pool = test_db().await.unwrap().pool();
    let exec = ExecutorHolder::new(pool);
    let res = sqlx::query!("SELECT 2 + 2 as value")
      .map(|rec| rec.value)
      .fetch_one(&mut exec.borrow())
      .await
      .unwrap();
    assert_eq!(4, res);
  }

  #[tokio::test]
  async fn should_execute_query_by_shared_txn() {
    let pool = test_db().await.unwrap().pool();
    begin(pool.clone(), async move {
      let exec = ExecutorHolder::new(pool);
      let res = sqlx::query!("SELECT 2 + 2 as value")
        .map(|rec| rec.value)
        .fetch_one(&mut exec.borrow())
        .await
        .unwrap();
      assert_eq!(4, res);
      commit().await.unwrap();
    })
    .await
    .unwrap();
  }
}
