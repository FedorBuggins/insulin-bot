pub mod event_publisher;
pub mod send_payload;

use teloxide::{
  dispatching::UpdateFilterExt,
  types::{Message, Update, User},
};

use crate::app::UpdateHandler;

pub fn filter_message() -> UpdateHandler {
  Update::filter_message()
    .filter_map(|msg: Message| msg.from().cloned())
    .map(|msg: Message| msg.id)
    .map(|msg: Message| msg.chat.id)
    .map(|user: User| user.id)
}
