use teloxide::types::UserId;

use crate::db::{txn::ExecutorHolder, Db};

use super::{Insulin, InsulinInjection};

pub fn insulin_injections(db: &Db, user_id: UserId) -> Repository {
  let exec = db.exec();
  Repository { user_id, exec }
}

pub struct Repository {
  user_id: UserId,
  exec: ExecutorHolder,
}

impl Repository {
  pub async fn fetch_all(
    &self,
  ) -> sqlx::Result<Vec<InsulinInjection>> {
    let user_id = self.user_id();
    sqlx::query!(
      r#"
        SELECT date_time, cubic_centimeters
        FROM insulin_injections
        WHERE user_id = ?
      "#,
      user_id
    )
    .map(|rec| InsulinInjection {
      date_time: rec.date_time.and_utc(),
      volume: Insulin::from_cubic_centimeters(rec.cubic_centimeters),
    })
    .fetch_all(&mut self.exec.borrow())
    .await
  }

  fn user_id(&self) -> i64 {
    self.user_id.0.try_into().unwrap()
  }

  pub async fn add(
    &mut self,
    insulin_injection: InsulinInjection,
  ) -> sqlx::Result<()> {
    let user_id = self.user_id();
    let date_time = insulin_injection.date_time;
    let cubic_centimeters =
      insulin_injection.volume.as_cubic_centimeters();
    sqlx::query!(
      r#"
        INSERT INTO insulin_injections (
          user_id,
          date_time,
          cubic_centimeters
        )
        VALUES (?, ?, ?)
      "#,
      user_id,
      date_time,
      cubic_centimeters
    )
    .execute(&mut self.exec.borrow())
    .await?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    app::user::repository::users,
    db::{tests::test_db, txn},
  };

  use super::*;

  #[tokio::test]
  async fn add_and_fetch_all() {
    let test_db = test_db().await.unwrap();
    txn::begin(test_db.pool(), async {
      let user = UserId(1);
      let insulin = Insulin::from_cubic_centimeters(5.7);
      let rec = InsulinInjection::from_now(insulin);
      users(&test_db).add(user).await.unwrap();
      let mut injections = insulin_injections(&test_db, user);
      injections.add(rec).await.unwrap();
      let recs = injections.fetch_all().await.unwrap();
      assert_eq!(recs, vec![rec]);
    })
    .await
    .unwrap();
  }
}
