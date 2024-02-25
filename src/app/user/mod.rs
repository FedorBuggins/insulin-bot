pub mod repository;

use std::sync::Arc;

use teloxide::{dispatching::HandlerExt, types::UserId};

use crate::{
  app, bot_commands::StartCommand, db::Db, utils::filter_message,
  UpdateHandler,
};

use self::repository::users;

pub struct Plugin;

impl app::Plugin for Plugin {
  fn update_handler(&self) -> UpdateHandler {
    filter_message().filter_command::<StartCommand>().endpoint(
      |db: Arc<Db>, user_id: UserId| async move {
        users(&db).add(user_id).await?;
        Ok(())
      },
    )
  }
}
