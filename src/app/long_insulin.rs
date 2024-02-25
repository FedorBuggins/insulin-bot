use std::sync::Arc;

use clokwerk::{AsyncScheduler, Job, TimeUnits};

use crate::{
  app,
  event_handler::{filter_event, handler, EventHandler},
  utils::event_publisher::EventPublisher,
};

#[derive(Debug, Clone)]
struct LongInsulinReminderScheduled;

pub struct Plugin;

impl app::Plugin for Plugin {
  fn event_handler(&self) -> EventHandler {
    filter_event::<LongInsulinReminderScheduled>().chain(handler(
      |event: LongInsulinReminderScheduled| async move {
        log::info!("{event:?} scheduled");
        Ok(())
      },
    ))
  }

  fn schedule(
    &self,
    scheduler: &mut AsyncScheduler,
    event_publisher: Arc<EventPublisher>,
  ) {
    scheduler.every(1.day()).plus(12.hours()).run(move || {
      event_publisher.send(LongInsulinReminderScheduled);
      async {}
    });
  }
}
