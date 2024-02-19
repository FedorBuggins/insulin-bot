use teloxide::types::UserId;

use super::txn::ExecutorHolder;

pub struct UserRepository {
  exec: ExecutorHolder,
}

impl UserRepository {
  #[must_use]
  pub(super) fn new(exec: ExecutorHolder) -> Self {
    Self { exec }
  }

  pub async fn fetch_all(&self) -> sqlx::Result<Vec<UserId>> {
    sqlx::query!("SELECT id FROM users WHERE disabled = FALSE")
      .map(|rec| UserId(rec.id.try_into().unwrap()))
      .fetch_all(&mut self.exec.borrow())
      .await
  }

  pub async fn add(&mut self, user_id: UserId) -> sqlx::Result<()> {
    let user_id: i64 = user_id.0.try_into().unwrap();
    sqlx::query!(
      "REPLACE INTO users (id, disabled) VALUES (?, FALSE)",
      user_id
    )
    .execute(&mut self.exec.borrow())
    .await?;
    Ok(())
  }
}
