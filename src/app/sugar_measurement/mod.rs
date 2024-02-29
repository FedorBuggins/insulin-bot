pub mod repository;

use std::sync::Arc;

use chrono::{DateTime, Utc};
use teloxide::{
  dispatching::dialogue::InMemStorage, dptree::case, prelude::*,
};

use crate::{
  app,
  bot_commands::MenuCommand,
  common::{any, Result},
  db::Db,
  utils::filter_message,
};

use self::repository::sugar_measurements;

use super::UpdateHandler;

type Dialog = Dialogue<State, InMemStorage<State>>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SugarMeasurement {
  pub date_time: DateTime<Utc>,
  pub level: SugarLevel,
}

impl SugarMeasurement {
  pub fn from_now(level: SugarLevel) -> Self {
    let date_time = Utc::now();
    Self { date_time, level }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SugarLevel {
  millimoles_per_liter: f64,
}

impl SugarLevel {
  pub fn from_millimoles_per_liter(
    millimoles_per_liter: f64,
  ) -> Self {
    Self {
      millimoles_per_liter,
    }
  }

  pub fn as_millimoles_per_liter(self) -> f64 {
    self.millimoles_per_liter
  }
}

#[derive(Default, Clone)]
enum State {
  #[default]
  Ignoring,
  Accepting,
}

pub struct Plugin;

impl app::Plugin for Plugin {
  fn prepare(&self, di: &mut DependencyMap) {
    di.insert(InMemStorage::<State>::new());
  }

  fn update_handler(&self) -> UpdateHandler {
    filter_message()
      .enter_dialogue::<Message, InMemStorage<State>, State>()
      .branch(
        dptree::entry()
          .filter_command::<MenuCommand>()
          .branch(case![MenuCommand::SugarLevel].endpoint(ask)),
      )
      .branch(case![State::Accepting].endpoint(accept))
  }
}

async fn ask(
  bot: Bot,
  chat_id: ChatId,
  dialogue: Dialog,
) -> Result<()> {
  bot
    .send_message(chat_id, "Отправьте уровень сахара в ммоль/л")
    .await?;
  dialogue.update(State::Accepting).await.map_err(any)?;
  Ok(())
}

async fn accept(
  bot: Bot,
  msg: Message,
  user_id: UserId,
  db: Arc<Db>,
  dialogue: Dialog,
) -> Result<()> {
  if let Some(sugar_level) = parse(msg.text()) {
    sugar_measurements(&db, user_id)
      .add(SugarMeasurement::from_now(sugar_level))
      .await?;
    bot.send_message(msg.chat.id, "✅").await?;
    dialogue.reset().await.map_err(any)?;
  } else {
    bot
      .send_message(msg.chat.id, "Неправильный формат (todo)")
      .await?;
  }
  Ok(())
}

fn parse(s: Option<&str>) -> Option<SugarLevel> {
  Some(SugarLevel::from_millimoles_per_liter(s?.parse().ok()?))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_5_dot_7() {
    let v = parse(Some("5.7"));
    let expected = Some(SugarLevel::from_millimoles_per_liter(5.7));
    assert_eq!(expected, v);
  }
}
