use std::sync::Arc;

use teloxide::types::UserId;

use crate::{common::Result, db::Db};

pub async fn add_user(db: Arc<Db>, user_id: UserId) -> Result<()> {
  db.users().add(user_id).await?;
  Ok(())
}
