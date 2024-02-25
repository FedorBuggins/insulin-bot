use teloxide::types::UserId;

use crate::db::{txn::ExecutorHolder, Db};

pub fn users(db: &Db) -> Repository {
  Repository { exec: db.exec() }
}

pub struct Repository {
  exec: ExecutorHolder,
}

impl Repository {
  #[allow(clippy::cast_sign_loss)]
  pub async fn fetch_all(&self) -> sqlx::Result<Vec<UserId>> {
    sqlx::query!("SELECT id FROM users WHERE disabled = FALSE")
      .map(|rec| UserId(rec.id as _))
      .fetch_all(&mut self.exec.borrow())
      .await
  }

  /// Registers user at system. Reactivates user if disabled.
  ///
  /// # Implementation details
  ///
  /// Replaces user record with order change
  #[allow(clippy::cast_possible_wrap)]
  pub async fn add(&mut self, user_id: UserId) -> sqlx::Result<()> {
    let user_id = user_id.0 as i64;
    sqlx::query!(
      "REPLACE INTO users (id, disabled) VALUES (?, FALSE)",
      user_id
    )
    .execute(&mut self.exec.borrow())
    .await?;
    Ok(())
  }
}
