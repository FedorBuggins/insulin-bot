mod app;
mod bot_commands;
mod common;
mod db;
mod event_handler;
mod logging;
mod schedules;
mod utils;

use std::{error::Error, sync::Arc};

use dotenv::dotenv;
use teloxide::{
  dispatching::{DpHandlerDescription, HandlerExt, UpdateFilterExt},
  dptree::{
    case,
    di::{Asyncify, Injectable},
    entry,
  },
  prelude::*,
  types::User,
  update_listeners::Polling,
  utils::command::BotCommands,
};

use crate::{
  bot_commands::MenuCommand, common::Result,
  utils::event_publisher::EventPublisher,
};

type UpdateHandler =
  Handler<'static, DependencyMap, Result<()>, DpHandlerDescription>;

fn main() -> Result<(), Box<dyn Error>> {
  dotenv()?;
  logging::init();
  tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()?
    .block_on(launch_bot())
}

async fn launch_bot() -> Result<(), Box<dyn Error>> {
  log::info!("Starting bot ..");
  let bot = Bot::from_env();

  log::debug!("Update commands ..");
  bot.set_my_commands(MenuCommand::bot_commands()).await?;

  log::debug!("Connect database ..");
  let db = Arc::new(db::connect().await?);

  log::debug!("Prepare dependency injector ..");
  let ep = Arc::new(EventPublisher::new());
  let me = bot.get_me().await?;
  let mut di = dptree::deps![bot.clone(), db, ep, me];

  app::sugar_measurement::prepare(&mut di);

  log::debug!("Start event handlers ..");
  event_handler::init(di.clone());

  log::debug!("Start schedules ..");
  Asyncify(schedules::init).inject(&di)().await;

  logging::log_toast("Bot started 🎉");
  init_dispatcher(bot, di).await?;

  logging::log_toast("Bot stopped 🏁");
  Ok(())
}

async fn init_dispatcher(bot: Bot, di: DependencyMap) -> Result<()> {
  let polling = Polling::builder(bot.clone()).build();
  let handler = entry()
    .inspect(logging::log_update)
    .branch(update_handler());
  Dispatcher::builder(bot, handler)
    .dependencies(di)
    .default_handler(logging::log_unhandled_update)
    .enable_ctrlc_handler()
    .build()
    .dispatch_with_listener(polling, logging::for_update_listener())
    .await;
  Ok(())
}

fn update_handler() -> UpdateHandler {
  dptree::entry()
    .branch(app::user::update_handler())
    .branch(app::sugar_measurement::update_handler())
    .branch(
      filter_message().branch(
        dptree::entry()
          .filter_command::<MenuCommand>()
          .branch(case![MenuCommand::Help].endpoint(send_help)),
      ),
    )
}

fn filter_message() -> UpdateHandler {
  Update::filter_message()
    .filter_map(|msg: Message| msg.from().cloned())
    .map(|msg: Message| msg.id)
    .map(|msg: Message| msg.chat.id)
    .map(|user: User| user.id)
}

async fn send_help(bot: Bot, chat_id: ChatId) -> Result<()> {
  bot.send_message(chat_id, "Help (todo)").await?;
  Ok(())
}
