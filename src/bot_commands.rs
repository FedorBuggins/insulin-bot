use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "snake_case")]
pub enum StartCommand {
  Start,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "snake_case")]
pub enum MenuCommand {
  #[command(description = "Инструкция")]
  Help,
  #[command(description = "Указать уровень сахара")]
  SugarLevel,
}
