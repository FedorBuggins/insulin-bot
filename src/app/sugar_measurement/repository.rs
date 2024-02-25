use teloxide::types::UserId;

use crate::db::{txn::ExecutorHolder, Db};

use super::{SugarLevel, SugarMeasurement};

pub fn sugar_measurements(db: &Db, user_id: UserId) -> Repository {
  let exec = db.exec();
  Repository { user_id, exec }
}

pub struct Repository {
  user_id: UserId,
  exec: ExecutorHolder,
}

impl Repository {
  #[allow(clippy::cast_possible_truncation)]
  pub async fn fetch_all(
    &self,
  ) -> sqlx::Result<Vec<SugarMeasurement>> {
    let user_id = self.user_id();
    sqlx::query!(
      r#"
        SELECT date_time, millimoles_per_liter
        FROM sugar_measurements
        WHERE user_id = ?
      "#,
      user_id
    )
    .map(|rec| SugarMeasurement {
      date_time: rec.date_time.and_utc(),
      level: SugarLevel::from_millimoles_per_liter(
        rec.millimoles_per_liter as _,
      ),
    })
    .fetch_all(&mut self.exec.borrow())
    .await
  }

  #[allow(clippy::cast_possible_wrap)]
  fn user_id(&self) -> i64 {
    self.user_id.0 as i64
  }

  pub async fn add(
    &mut self,
    sugar_measurement: SugarMeasurement,
  ) -> sqlx::Result<()> {
    let user_id = self.user_id();
    let date_time = sugar_measurement.date_time;
    let millimoles_per_liter =
      sugar_measurement.level.as_millimoles_per_liter();
    sqlx::query!(
      r#"
        INSERT INTO sugar_measurements (
          user_id,
          date_time,
          millimoles_per_liter
        )
        VALUES (?, ?, ?)
      "#,
      user_id,
      date_time,
      millimoles_per_liter
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
  async fn save_5_dot_7() {
    let test_db = test_db().await.unwrap();
    txn::begin(test_db.pool(), async {
      let user = UserId(1);
      let sugar_level = SugarLevel::from_millimoles_per_liter(5.7);
      let rec = SugarMeasurement::from_now(sugar_level);
      users(&test_db).add(user).await.unwrap();
      let mut measurements = sugar_measurements(&test_db, user);
      measurements.add(rec).await.unwrap();
      let recs = measurements.fetch_all().await.unwrap();
      assert_eq!(recs, vec![rec]);
    })
    .await
    .unwrap();
  }
}
