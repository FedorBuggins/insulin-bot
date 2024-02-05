use std::{env, sync::Arc};

use lib::{
  auth::Auth, common::Result, db::JsonCell,
  utils::send_payload::SendPayload,
};

use teloxide::{
  dispatching::{HandlerExt, UpdateFilterExt},
  dptree::{self, case},
  macros::BotCommands,
  payloads::{SendChatAction, SendMessage, SendMessageSetters},
  requests::Requester,
  types::{
    ChatAction, ChatId, KeyboardButton, KeyboardMarkup,
    KeyboardRemove, Message, Update,
  },
  Bot,
};
use tokio::sync::RwLock;

use crate::{active_chats, UpdatesHandler};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "snake_case")]
enum AuthCommand {
  Auth,
  Allow(i64),
  Deny(i64),
}

impl AuthCommand {
  fn message(&self) -> String {
    match self {
      AuthCommand::Auth => "/auth".to_string(),
      AuthCommand::Allow(chat_id) => format!("/allow {chat_id}"),
      AuthCommand::Deny(chat_id) => format!("/deny {chat_id}"),
    }
  }
}

pub async fn init(
  store: Arc<JsonCell<Auth>>,
) -> Result<UpdatesHandler> {
  let auth = _init(store).await?;
  let handler = dptree::map(move || auth.clone())
    .branch(
      Update::filter_message()
        .filter_command::<AuthCommand>()
        .branch(admin_commands())
        .branch(case![AuthCommand::Auth].endpoint(request_auth)),
    )
    .filter_async(is_allowed_chat);
  Ok(handler)
}

async fn _init(
  store: Arc<JsonCell<Auth>>,
) -> Result<Arc<RwLock<Auth>>> {
  let admin = env::var("ADMIN").unwrap();
  let sign = env::var("AUTH_SIGN").unwrap();
  let auth = match store.get().await? {
    Some(auth) if auth.is_actual(&admin, &sign) => auth,
    _ => Auth::new(admin, sign),
  };
  update_active_chats(&auth);
  Ok(Arc::new(RwLock::new(auth)))
}

fn update_active_chats(auth: &Auth) {
  active_chats::set(auth.allowed_chats().clone());
}

fn admin_commands() -> UpdatesHandler {
  dptree::filter_map_async(filter_admin_chat_id)
    .branch(case![AuthCommand::Auth].endpoint(set_admin_chat_id))
    .branch(
      case![AuthCommand::Allow(chat_id)]
        .map(|chat_id: i64| ChatId(chat_id))
        .endpoint(allow_chat),
    )
    .branch(
      case![AuthCommand::Deny(chat_id)]
        .map(|chat_id: i64| ChatId(chat_id))
        .endpoint(deny_chat),
    )
}

async fn filter_admin_chat_id(
  msg: Message,
  auth: Arc<RwLock<Auth>>,
) -> Option<ChatId> {
  let username = msg.from().as_ref()?.username.as_ref()?;
  let auth = auth.read().await;
  let admin_username = &auth.admin().username;
  if username == admin_username && msg.chat.is_private() {
    Some(msg.chat.id)
  } else {
    None
  }
}

async fn set_admin_chat_id(
  store: Arc<JsonCell<Auth>>,
  bot: Bot,
  chat_id: ChatId,
  auth: Arc<RwLock<Auth>>,
) -> Result<()> {
  let auth = &mut auth.write().await;
  auth.set_admin_chat_id(chat_id);
  store.save(auth).await?;
  bot.send_message(chat_id, "Добро пожаловать, босс!").await?;
  Ok(())
}

async fn request_auth(
  msg: Message,
  auth: Arc<RwLock<Auth>>,
  bot: Bot,
) -> Result<()> {
  let admin_chat_id = auth.read().await.admin().chat_id;
  let admin_chat_id = admin_chat_id.ok_or("No admin chat id yet")?;
  let chat_id = msg.chat.id;
  let chat =
    msg.chat.title().map_or(format!("{chat_id:?}"), <_>::into);
  let text = format!("Разрешить доступ для {chat}?");
  SendMessage::new(admin_chat_id, text)
    .reply_markup(KeyboardMarkup::new([[
      AuthCommand::Allow(chat_id.0),
      AuthCommand::Deny(chat_id.0),
    ]
    .map(|cmd| KeyboardButton::new(cmd.message()))]))
    .send_by(bot.clone())
    .await?;
  SendChatAction::new(chat_id, ChatAction::Typing)
    .send_by(bot)
    .await?;
  Ok(())
}

async fn allow_chat(
  store: Arc<JsonCell<Auth>>,
  chat_id: ChatId,
  auth: Arc<RwLock<Auth>>,
  bot: Bot,
) -> Result<()> {
  let auth = &mut auth.write().await;
  let admin_chat_id =
    auth.admin().chat_id.ok_or("No admin chat id yet")?;
  auth.allow(chat_id);
  update_active_chats(auth);
  store.save(auth).await?;
  bot.send_message(chat_id, "Здравия желаю!").await?;
  SendMessage::new(admin_chat_id, "Чат авторизован")
    .reply_markup(KeyboardRemove::new())
    .send_by(bot)
    .await?;
  Ok(())
}

async fn deny_chat(
  store: Arc<JsonCell<Auth>>,
  chat_id: ChatId,
  auth: Arc<RwLock<Auth>>,
  bot: Bot,
) -> Result<()> {
  let auth = &mut auth.write().await;
  let admin_chat_id =
    auth.admin().chat_id.ok_or("No admin chat id yet")?;
  auth.deny(chat_id);
  update_active_chats(auth);
  store.save(auth).await?;
  SendMessage::new(admin_chat_id, "Чат заблокирован")
    .reply_markup(KeyboardRemove::new())
    .send_by(bot)
    .await?;
  Ok(())
}

async fn is_allowed_chat(
  auth: Arc<RwLock<Auth>>,
  upd: Update,
) -> bool {
  async move {
    let chat = upd.chat()?;
    let auth = auth.read().await;
    if auth.allowed_chats().contains(&chat.id) {
      Some(true)
    } else if chat.is_private() {
      let username = upd.user()?.username.as_ref()?;
      Some(*username == auth.admin().username)
    } else {
      Some(false)
    }
  }
  .await
  .unwrap_or_default()
}
