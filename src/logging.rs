pub mod critical;

use std::{
  env, fmt,
  future::Future,
  io::{self, Write},
  pin::Pin,
  process::Command,
  sync::{Arc, Mutex},
  time::Duration,
};

use chrono::Local;
use teloxide::{
  error_handlers::ErrorHandler,
  types::{Update, UpdateKind},
};
use tokio::time::sleep;

static LAST_LOG: Mutex<(usize, String)> =
  Mutex::new((0, String::new()));

enum LogType {
  NewLog { after_same_error: bool },
  SameError(usize),
}

pub fn init() {
  let filters =
    env::var("RUST_LOG").unwrap_or(log::Level::Info.to_string());
  env_logger::builder()
    .parse_filters(&filters)
    .format(write_log)
    .init();
}

fn write_log(
  f: &mut env_logger::fmt::Formatter,
  rec: &log::Record,
) -> io::Result<()> {
  let level = rec.level();
  let level_sign = level_sign(level);
  let time = Local::now().format("%T%.3f");
  let prefix = format!("{level_sign} [{time}]");
  let args = format!("{}", rec.args());
  match handle_log(level, &args) {
    LogType::NewLog { after_same_error } => {
      let br = if after_same_error { "\n\n" } else { "" };
      writeln!(f, "{br}{prefix} > {args}\n")
    }
    LogType::SameError(n) => {
      write!(f, "{prefix} > Same error ({n})\r")
    }
  }
}

fn handle_log(level: log::Level, args: &str) -> LogType {
  let mut last_log = LAST_LOG.lock().unwrap();
  let error = level == log::Level::Error;
  let same_error = last_log.1 == args;
  let after_same_error = last_log.0 > 0;
  if same_error {
    last_log.0 += 1;
  } else if error {
    *last_log = (0, args.to_string());
  } else {
    *last_log = Default::default();
  }
  if same_error {
    LogType::SameError(last_log.0)
  } else {
    LogType::NewLog { after_same_error }
  }
}

fn level_sign(level: log::Level) -> &'static str {
  match level {
    log::Level::Error => "🔴",
    log::Level::Warn => "🟠",
    log::Level::Info => "🟢",
    log::Level::Debug => "🔵",
    log::Level::Trace => "🟣",
  }
}

#[allow(clippy::needless_pass_by_value)]
pub fn log_update(upd: Update) {
  let upd_kind = upd_kind_name(&upd);
  let upd_id = upd.id;
  let chat_name =
    chat_title_or_id(&upd).unwrap_or("<unknown>".to_string());
  log::info!("Get {upd_kind} #{upd_id} from {chat_name}");
  log::debug!("{upd:?}");
}

fn chat_title_or_id(upd: &Update) -> Option<String> {
  let chat = upd.chat()?;
  if let Some(title) = chat.title() {
    Some(title.to_string())
  } else {
    Some(chat.id.to_string())
  }
}

fn upd_kind_name(upd: &Update) -> &str {
  match upd.kind {
    UpdateKind::Message(_) => "Message",
    UpdateKind::EditedMessage(_) => "EditedMessage",
    UpdateKind::ChannelPost(_) => "ChannelPost",
    UpdateKind::EditedChannelPost(_) => "EditedChannelPost",
    UpdateKind::InlineQuery(_) => "InlineQuery",
    UpdateKind::ChosenInlineResult(_) => "ChosenInlineResult",
    UpdateKind::CallbackQuery(_) => "CallbackQuery",
    UpdateKind::ShippingQuery(_) => "ShippingQuery",
    UpdateKind::PreCheckoutQuery(_) => "PreCheckoutQuery",
    UpdateKind::Poll(_) => "Poll",
    UpdateKind::PollAnswer(_) => "PollAnswer",
    UpdateKind::MyChatMember(_) => "MyChatMember",
    UpdateKind::ChatMember(_) => "ChatMember",
    UpdateKind::ChatJoinRequest(_) => "ChatJoinRequest",
    UpdateKind::Error(_) => "Error",
  }
}

pub async fn log_unhandled_update(upd: Arc<Update>) {
  let upd_kind = upd_kind_name(&upd);
  let upd_id = upd.id;
  let chat_name =
    chat_title_or_id(&upd).unwrap_or("<unknown>".to_string());
  log::warn!("Unhandled {upd_kind} #{upd_id} from {chat_name}");
}

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub struct UpdateListenerErrorHandler;

impl UpdateListenerErrorHandler {
  const DELAY: Duration = Duration::from_secs(2);
}

impl<E> ErrorHandler<E> for UpdateListenerErrorHandler
where
  E: fmt::Debug + fmt::Display,
{
  fn handle_error(self: Arc<Self>, err: E) -> BoxFuture<'static, ()> {
    log::error!("{err}");
    log::debug!("Sleep {} sec ..", Self::DELAY.as_secs());
    Box::pin(sleep(Self::DELAY))
  }
}

pub fn for_update_listener() -> Arc<UpdateListenerErrorHandler> {
  UpdateListenerErrorHandler.into()
}

pub fn log_toast(text: &str) {
  log::info!("{text}");
  toast(text);
}

fn toast(text: &str) {
  const DARK_GREEN: &str = "#1e6c30";
  let _ = Command::new("termux-toast")
    .args(["-g", "top"])
    .args(["-b", DARK_GREEN])
    .arg(text)
    .spawn();
}
