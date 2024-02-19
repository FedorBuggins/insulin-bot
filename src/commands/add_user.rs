use std::sync::Arc;

use lib::{common::Result, db::Db};
use teloxide::types::UserId;

pub async fn add_user(db: Arc<Db>, user_id: UserId) -> Result<()> {
  db.users().add(user_id).await?;
  Ok(())
}
