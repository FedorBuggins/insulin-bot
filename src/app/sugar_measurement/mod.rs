pub mod repository;

use std::sync::Arc;

use chrono::{DateTime, Utc};
use teloxide::{
  dispatching::dialogue::InMemStorage, dptree::case, prelude::*,
};

use crate::{
  app, bot_commands::MenuCommand, common::Result, db::Db,
  utils::filter_message, UpdateHandler,
};

use self::repository::sugar_measurements;

type Dialog = Dialogue<State, InMemStorage<State>>;

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

pub struct SugarLevel {
  millimoles_per_liter: f32,
}

impl SugarLevel {
  pub fn from_millimoles_per_liter(
    millimoles_per_liter: f32,
  ) -> Self {
    Self {
      millimoles_per_liter,
    }
  }

  pub fn as_millimoles_per_liter(&self) -> f32 {
    self.millimoles_per_liter
  }
}

#[derive(Default, Clone)]
enum State {
  #[default]
  Default,
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
      .branch(dptree::entry().filter_command::<MenuCommand>().branch(
        case![MenuCommand::SugarLevel].endpoint(prepare_accepting),
      ))
      .branch(case![State::Accepting].endpoint(accept))
  }
}

async fn prepare_accepting(
  bot: Bot,
  chat_id: ChatId,
  dialogue: Dialog,
) -> Result<()> {
  bot
    .send_message(chat_id, "Отправьте уровень сахара в ммоль/л")
    .await?;
  dialogue.update(State::Accepting).await.unwrap();
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
  } else {
    bot
      .send_message(msg.chat.id, "Неправильный формат (todo)")
      .await?;
  }
  dialogue.reset().await.unwrap();
  Ok(())
}

fn parse(s: Option<&str>) -> Option<SugarLevel> {
  Some(SugarLevel::from_millimoles_per_liter(s?.parse().ok()?))
}
