pub mod repository;

use std::sync::Arc;

use chrono::{DateTime, Utc};
use teloxide::{
  dispatching::dialogue::{Dialogue, InMemStorage},
  dptree::case,
  prelude::*,
};

use crate::{
  bot_commands::MenuCommand, common::Result, db::Db, filter_message,
  UpdateHandler,
};

use self::repository::sugar_measurements;

type SugarLevelDialogue = Dialogue<
  SugarLevelDialogueState,
  InMemStorage<SugarLevelDialogueState>,
>;

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
enum SugarLevelDialogueState {
  #[default]
  None,
  Accepting,
}

pub fn prepare(di: &mut DependencyMap) {
  di.insert(InMemStorage::<SugarLevelDialogueState>::new());
}

pub fn update_handler() -> UpdateHandler {
  filter_message()
    .enter_dialogue::<Message, InMemStorage<SugarLevelDialogueState>, SugarLevelDialogueState>()
    .branch(
      dptree::entry().filter_command::<MenuCommand>().branch(
        case![MenuCommand::SugarLevel]
          .endpoint(prepare_sugar_level_accepting),
      ),
    )
    .branch(
      case![SugarLevelDialogueState::Accepting]
        .endpoint(accept_sugar_level),
    )
}

async fn prepare_sugar_level_accepting(
  bot: Bot,
  chat_id: ChatId,
  dialogue: SugarLevelDialogue,
) -> Result<()> {
  bot
    .send_message(chat_id, "Отправьте уровень сахара в ммоль/л")
    .await?;
  dialogue
    .update(SugarLevelDialogueState::Accepting)
    .await
    .unwrap();
  Ok(())
}

async fn accept_sugar_level(
  bot: Bot,
  msg: Message,
  user_id: UserId,
  db: Arc<Db>,
  dialogue: SugarLevelDialogue,
) -> Result<()> {
  if let Some(sugar_level) = parse_sugar_level(msg.text()) {
    sugar_measurements(&db, user_id)
      .add(SugarMeasurement::from_now(sugar_level))
      .await?;
  } else {
    bot
      .send_message(msg.chat.id, "Неправильный формат (todo)")
      .await?;
  }
  dialogue.reset().await.unwrap();
  Ok(())
}

fn parse_sugar_level(s: Option<&str>) -> Option<SugarLevel> {
  Some(SugarLevel::from_millimoles_per_liter(s?.parse().ok()?))
}
