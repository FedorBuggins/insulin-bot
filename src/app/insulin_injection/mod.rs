mod repository;

use std::sync::Arc;

use chrono::{DateTime, Utc};
use teloxide::{
  dispatching::{
    dialogue::{Dialogue, InMemStorage},
    HandlerExt,
  },
  dptree::{self, case, di::DependencyMap},
  requests::Requester,
  types::{ChatId, Message, UserId},
  Bot,
};

use crate::{
  bot_commands::MenuCommand,
  common::{any, Result},
  db::Db,
  utils::filter_message,
};

use self::repository::insulin_injections;

use super::UpdateHandler;

pub struct Plugin;

impl super::Plugin for Plugin {
  fn prepare(&self, di: &mut DependencyMap) {
    di.insert(InMemStorage::<State>::new());
  }

  fn update_handler(&self) -> UpdateHandler {
    filter_message()
      .enter_dialogue::<Message, InMemStorage<State>, State>()
      .branch(
        dptree::entry()
          .filter_command::<MenuCommand>()
          .branch(case![MenuCommand::InsulinInjection].endpoint(ask)),
      )
      .branch(case![State::Accepting].endpoint(accept))
  }
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct InsulinInjection {
  date_time: DateTime<Utc>,
  volume: Insulin,
}

impl InsulinInjection {
  fn from_now(volume: Insulin) -> Self {
    let date_time = Utc::now();
    Self { date_time, volume }
  }
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Insulin {
  cubic_centimeters: f64,
}

impl Insulin {
  fn from_cubic_centimeters(cubic_centimeters: f64) -> Self {
    Self { cubic_centimeters }
  }

  fn as_cubic_centimeters(self) -> f64 {
    self.cubic_centimeters
  }
}

#[derive(Default, Clone)]
enum State {
  #[default]
  Ignoring,
  Accepting,
}

type Dialog = Dialogue<State, InMemStorage<State>>;

async fn ask(
  bot: Bot,
  chat_id: ChatId,
  dialogue: Dialog,
) -> Result<()> {
  bot
    .send_message(chat_id, "Отправьте инсулин в см³ (ЕД)")
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
  if let Some(insulin) = parse(msg.text()) {
    insulin_injections(&db, user_id)
      .add(InsulinInjection::from_now(insulin))
      .await?;
    bot.send_message(msg.chat.id, "✅").await?;
  } else {
    bot
      .send_message(msg.chat.id, "Неправильный формат (todo)")
      .await?;
  }
  dialogue.reset().await.map_err(any)?;
  Ok(())
}

fn parse(s: Option<&str>) -> Option<Insulin> {
  Some(Insulin::from_cubic_centimeters(s?.parse().ok()?))
}
