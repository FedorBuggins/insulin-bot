use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "snake_case")]
pub enum BotCommand {
  #[command(description = "Инструкция")]
  Help,
}
