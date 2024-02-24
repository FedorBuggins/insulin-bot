pub mod repository;

use std::sync::Arc;

use teloxide::{dispatching::HandlerExt, types::UserId};

use crate::{
  bot_commands::StartCommand, db::Db, filter_message, UpdateHandler,
};

use self::repository::users;

pub fn update_handler() -> UpdateHandler {
  filter_message().filter_command::<StartCommand>().endpoint(
    |db: Arc<Db>, user_id: UserId| async move {
      users(&db).add(user_id).await?;
      Ok(())
    },
  )
}
