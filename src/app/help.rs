use teloxide::{dptree::case, prelude::*};

use crate::{
  app, bot_commands::MenuCommand, common::Result,
  utils::filter_message,
};

use super::UpdateHandler;

pub struct Plugin;

impl app::Plugin for Plugin {
  fn update_handler(&self) -> UpdateHandler {
    filter_message()
      .filter_command::<MenuCommand>()
      .branch(case![MenuCommand::Help].endpoint(send_help))
  }
}

async fn send_help(bot: Bot, chat_id: ChatId) -> Result<()> {
  bot.send_message(chat_id, "Help (todo)").await?;
  Ok(())
}
