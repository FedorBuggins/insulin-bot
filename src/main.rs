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
  dptree::di::{Asyncify, Injectable},
  prelude::*,
  utils::command::BotCommands,
};

use crate::{
  app::plugins, bot_commands::MenuCommand, common::Result,
  utils::event_publisher::EventPublisher,
};

fn main() -> Result<(), Box<dyn Error>> {
  dotenv()?;
  logging::init();
  tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()?
    .block_on(launch())
}

async fn launch() -> Result<(), Box<dyn Error>> {
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
  for plugin in plugins() {
    plugin.prepare(&mut di);
  }

  log::debug!("Start event handlers ..");
  event_handler::init(di.clone());

  log::debug!("Start schedules ..");
  Asyncify(schedules::init).inject(&di)().await;

  log::info!("Bot started 🎉");
  dispatch(bot, di).await;

  log::info!("Bot stopped 🏁");
  Ok(())
}

async fn dispatch(bot: Bot, di: DependencyMap) {
  let mut handler = dptree::entry().inspect(logging::log_update);
  for plugin in plugins() {
    handler = handler.branch(plugin.update_handler());
  }
  Dispatcher::builder(bot, handler)
    .dependencies(di)
    .default_handler(logging::log_unhandled_update)
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}
