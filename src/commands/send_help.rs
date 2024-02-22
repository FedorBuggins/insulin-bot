use teloxide::{requests::Requester, types::ChatId, Bot};

use crate::common::Result;

pub async fn send_help(bot: Bot, chat_id: ChatId) -> Result<()> {
  bot.send_message(chat_id, "Help (todo)").await?;
  Ok(())
}
