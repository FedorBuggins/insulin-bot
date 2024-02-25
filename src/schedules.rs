use std::{sync::Arc, time::Duration};

use clokwerk::AsyncScheduler;
use tokio::{task::AbortHandle, time::interval};

use crate::{app::plugins, utils::event_publisher::EventPublisher};

#[allow(clippy::needless_pass_by_value)]
pub fn init(ep: Arc<EventPublisher>) -> AbortHandle {
  let mut scheduler = AsyncScheduler::new();
  for plugin in plugins() {
    plugin.schedule(&mut scheduler, ep.clone());
  }
  tokio::spawn(async move {
    let mut interval = interval(Duration::from_secs(5));
    loop {
      interval.tick().await;
      scheduler.run_pending().await;
    }
  })
  .abort_handle()
}
