use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use teloxide::types::ChatId;

pub type Username = String;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Admin {
  pub username: Username,
  pub chat_id: Option<ChatId>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Auth {
  admin: Admin,
  sign: String,
  allowed_chats: HashSet<ChatId>,
}

impl Auth {
  pub fn new(
    admin: impl Into<Username>,
    sign: impl Into<String>,
  ) -> Self {
    Self {
      admin: Admin {
        username: admin.into(),
        chat_id: None,
      },
      sign: sign.into(),
      allowed_chats: HashSet::default(),
    }
  }

  #[must_use]
  pub fn is_actual(&self, admin: &Username, sign: &str) -> bool {
    self.admin.username == *admin && self.sign == sign
  }

  #[must_use]
  pub fn admin(&self) -> &Admin {
    &self.admin
  }

  pub fn set_admin_chat_id(&mut self, chat_id: ChatId) {
    self.admin.chat_id = Some(chat_id);
  }

  #[must_use]
  pub fn sign(&self) -> &String {
    &self.sign
  }

  #[must_use]
  pub fn allowed_chats(&self) -> &HashSet<ChatId> {
    &self.allowed_chats
  }

  pub fn allow(&mut self, chat_id: ChatId) {
    self.allowed_chats.insert(chat_id);
  }

  pub fn deny(&mut self, chat_id: ChatId) {
    self.allowed_chats.remove(&chat_id);
  }
}
