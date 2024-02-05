use std::{sync::Arc, time::Duration};

use chrono::Local;
use teloxide::{
  dptree::di::{DependencySupplier, Injectable},
  prelude::*,
};
use tokio::time::sleep;

use crate::{
  common::Result,
  logging,
  utils::event_publisher::{AnyEvent, Event, EventPublisher},
};

type EventsHandler = Handler<'static, DependencyMap, ()>;

pub fn init(di: DependencyMap) {
  let events_handler = dptree::entry().inspect(log_event_received);
  tokio::spawn(launch(events_handler, di));
}

#[allow(clippy::needless_pass_by_value)]
fn log_event_received(event: Event) {
  log::info!("Receive event {event:?}");
}

fn filter_event<T: AnyEvent>() -> EventsHandler {
  dptree::filter_map(|event: Event| event.downcast::<T>())
}

fn handler<F, A>(f: F) -> EventsHandler
where
  F: Injectable<DependencyMap, Result<()>, A> + Send + Sync + 'static,
  A: 'static,
{
  let f = Arc::new(f);
  dptree::from_fn(move |di: DependencyMap, next| {
    tokio::spawn(handle(f.clone(), di.clone()));
    next(di)
  })
}

async fn handle<F, A>(f: Arc<F>, di: DependencyMap)
where
  F: Injectable<DependencyMap, Result<()>, A> + Send + Sync + 'static,
{
  const MAX_ATTEMPTS: u32 = 4;
  let mut attempt = 0;
  while let Err(err) = f.inject(&di)().await {
    if err.is_network_problem() && attempt < MAX_ATTEMPTS {
      log::error!("{err}");
      log::info!("Retrying ..");
      sleep(Duration::from_secs(2u64.pow(attempt + 1))).await;
    } else {
      let event = DependencySupplier::<Event>::get(&di);
      let now = Local::now();
      logging::critical::error(format!(
        "\
          EVENT HANDLING FAILED!!!\n\n\
          {now}\n\n\
          Event: {event:?}\n\n\
          Error: {err}\n\n\
          {err:?}\
        "
      ));
      break;
    }
    attempt += 1;
  }
}

async fn launch(
  events_handler: EventsHandler,
  di: DependencyMap,
) -> ! {
  let events_handler = Arc::new(events_handler);
  let mut events =
    DependencySupplier::<Arc<EventPublisher>>::get(&di).subscribe();
  loop {
    let event = events.recv().await.unwrap();
    let mut di = di.clone();
    di.insert(event);
    tokio::spawn(dispatch(events_handler.clone(), di));
  }
}

async fn dispatch(
  events_handler: Arc<EventsHandler>,
  di: DependencyMap,
) {
  events_handler.dispatch(di).await;
}
