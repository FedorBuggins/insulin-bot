use std::{sync::Arc, time::Duration};

use clokwerk::{AsyncScheduler, Job, TimeUnits};
use tokio::{task::AbortHandle, time::interval};

use crate::utils::event_publisher::EventPublisher;

#[derive(Debug, Clone)]
pub struct LongInsulinReminderScheduled;

pub fn init(ep: Arc<EventPublisher>) -> AbortHandle {
  let mut scheduler = AsyncScheduler::new();
  schedule_report_requests_check(&mut scheduler, ep);
  tokio::spawn(async move {
    let mut interval = interval(Duration::from_secs(5));
    loop {
      interval.tick().await;
      scheduler.run_pending().await;
    }
  })
  .abort_handle()
}

fn schedule_report_requests_check(
  scheduler: &mut AsyncScheduler,
  event_publisher: Arc<EventPublisher>,
) {
  scheduler.every(1.day()).plus(12.hours()).run(move || {
    event_publisher.send(LongInsulinReminderScheduled);
    async {}
  });
}
