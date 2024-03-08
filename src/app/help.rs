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
    const HELP_MESSAGE: &str = "
Use this bot for:
- save and track your sugar measurements
- get reminders for an insulin injections
- see your average amount of insulin

List of commands:
  /help - print this message
  /sugar_level - send your last sugar measurement
  /insulin_injection - send your last insulin injection
";
    bot.send_message(chat_id, HELP_MESSAGE).await?;
    Ok(())
}
