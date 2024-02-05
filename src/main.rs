mod active_chats;
mod auth_proxy;
mod bot_commands;
mod commands;
mod env;
mod event_handlers;
mod logging;
mod schedules;

extern crate insulin_bot as lib;
use lib::{auth::Auth, common, db, utils};

use std::{error::Error, sync::Arc, time::Duration};

use teloxide::{
  dispatching::{DpHandlerDescription, UpdateFilterExt},
  dptree::{
    case,
    di::{Asyncify, Injectable},
    entry,
  },
  net,
  prelude::*,
  types::User,
  update_listeners::Polling,
  utils::command::BotCommands,
};

use crate::{
  bot_commands::BotCommand, common::Result,
  utils::event_publisher::EventPublisher,
};

const WORKER_THREADS: usize = 2;
const MAX_BLOCKING_THREADS: usize = 10;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(20);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(40);

type UpdatesHandler =
  Handler<'static, DependencyMap, Result<()>, DpHandlerDescription>;

fn main() -> Result<(), Box<dyn Error>> {
  env::init()?;
  logging::init();
  tokio::runtime::Builder::new_multi_thread()
    .worker_threads(WORKER_THREADS)
    .max_blocking_threads(MAX_BLOCKING_THREADS)
    .enable_all()
    .build()?
    .block_on(launch_bot())
}

async fn launch_bot() -> Result<(), Box<dyn Error>> {
  log::info!("Starting bot ..");
  let bot = Bot::from_env_with_client(
    net::default_reqwest_settings()
      .connect_timeout(CONNECT_TIMEOUT)
      .timeout(REQUEST_TIMEOUT)
      .https_only(true)
      .build()?,
  );

  log::debug!("Update commands ..");
  update_commands(&bot).await?;

  log::debug!("Connect database ..");
  let db = Arc::new(db::connect().await?);

  log::debug!("Prepare dependency injector ..");
  let ep = Arc::new(EventPublisher::new());
  let me = bot.get_me().await?;
  let auth_store = Arc::new(db.json_cell::<Auth>("auth"));
  let di = dptree::deps![bot.clone(), db, ep, me, auth_store];

  log::debug!("Start event handlers ..");
  event_handlers::init(di.clone());

  log::debug!("Start schedules ..");
  Asyncify(schedules::init).inject(&di)().await;

  logging::log_toast("Bot started 🎉");
  init_dispatcher(bot, di).await?;

  logging::log_toast("Bot stopped 🏁");
  Ok(())
}

async fn update_commands(bot: &Bot) -> Result<()> {
  const MAX_ATTEMPTS: u8 = 3;
  for attempt in 0.. {
    match bot.set_my_commands(BotCommand::bot_commands()).await {
      Ok(_) => break,
      Err(_) if attempt < MAX_ATTEMPTS => continue,
      Err(err) => Err(err)?,
    }
  }
  Ok(())
}

async fn init_dispatcher(bot: Bot, di: DependencyMap) -> Result<()> {
  let polling = Polling::builder(bot.clone())
    .timeout(REQUEST_TIMEOUT)
    .build();
  let handler = entry()
    .inspect(logging::log_update)
    .chain(auth_proxy::init.inject(&di)().await?)
    .branch(commands_handler());
  Dispatcher::builder(bot, handler)
    .dependencies(di)
    .default_handler(logging::log_unhandled_update)
    .worker_queue_size(MAX_BLOCKING_THREADS)
    .enable_ctrlc_handler()
    .build()
    .dispatch_with_listener(polling, logging::for_update_listener())
    .await;
  Ok(())
}

fn commands_handler() -> UpdatesHandler {
  filter_message()
    .filter_command::<BotCommand>()
    .branch(case![BotCommand::Help].endpoint(|| async { todo!() }))
}

fn filter_message() -> UpdatesHandler {
  Update::filter_message()
    .filter_map(|msg: Message| msg.from().cloned())
    .map(|msg: Message| msg.id)
    .map(|msg: Message| msg.chat.id)
    .map(|user: User| user.id)
}
