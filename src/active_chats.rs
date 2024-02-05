use std::sync::RwLock;

use teloxide::types::ChatId;

static ACTIVE_CHATS: RwLock<Vec<ChatId>> = RwLock::new(Vec::new());

pub fn get() -> Vec<ChatId> {
  match ACTIVE_CHATS.read() {
    Ok(chats) => chats.to_owned(),
    Err(err) => {
      log::error!("{err}");
      Vec::default()
    }
  }
}

pub fn set(chats: impl IntoIterator<Item = ChatId>) {
  match ACTIVE_CHATS.write() {
    Ok(mut cell) => *cell = chats.into_iter().collect(),
    Err(err) => log::error!("{err}"),
  }
}
