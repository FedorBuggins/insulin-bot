mod help;
mod long_insulin;
mod sugar_measurement;
mod user;

use std::sync::Arc;

use clokwerk::AsyncScheduler;
use teloxide::{
  dispatching::DpHandlerDescription,
  dptree::{self, di::DependencyMap, Handler},
};

use crate::{
  common::Result, event_handler::EventHandler,
  utils::event_publisher::EventPublisher,
};

pub type UpdateHandler =
  Handler<'static, DependencyMap, Result<()>, DpHandlerDescription>;

pub trait Plugin {
  fn prepare(&self, di: &mut DependencyMap) {
    let _ = di;
  }

  fn update_handler(&self) -> UpdateHandler {
    dptree::entry()
  }

  fn event_handler(&self) -> EventHandler {
    dptree::entry()
  }

  fn schedule(
    &self,
    scheduler: &mut AsyncScheduler,
    event_publisher: Arc<EventPublisher>,
  ) {
    let _ = scheduler;
    let _ = event_publisher;
  }
}

pub fn plugins() -> Vec<Box<dyn Plugin>> {
  vec![
    Box::new(help::Plugin),
    Box::new(long_insulin::Plugin),
    Box::new(sugar_measurement::Plugin),
    Box::new(user::Plugin),
  ]
}
