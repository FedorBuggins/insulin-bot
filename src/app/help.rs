use teloxide::{
  dptree::case, prelude::*, utils::command::BotCommands,
};

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
  let help_message = format!("
Этот бот имеет следующие возможности:
- сохранение и просмотр ваших показаний уровня сахара
- отправка напоминаний о необходимости измерения сахара и/или инъекции инсулина
- просмотр среднего количества инсулина за период времени.

Функциональность постепенно увеличивается, бот находится в активной разработке. 

Список доступных команд:
{}", MenuCommand::descriptions());

  bot.send_message(chat_id, help_message).await?;
  Ok(())
}
