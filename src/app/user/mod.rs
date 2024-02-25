pub mod repository;

use std::sync::Arc;

use teloxide::{dispatching::HandlerExt, types::UserId};

use crate::{
  app, bot_commands::StartCommand, db::Db, utils::filter_message,
};

use self::repository::users;

use super::UpdateHandler;

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
