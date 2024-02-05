pub mod event_bus;

use std::{
  any::Any,
  fmt::{self, Debug},
  sync::Arc,
};

use tokio::sync::broadcast;

pub trait AnyEvent: Any + Debug + Clone + Send + Sync {}

impl<T: Any + Debug + Clone + Send + Sync> AnyEvent for T {}

#[derive(Clone)]
pub struct Event {
  debug: String,
  inner: Arc<dyn Any + Send + Sync>,
}

impl Event {
  fn new(inner: impl AnyEvent) -> Self {
    let debug = format!("{inner:?}");
    let inner = Arc::new(inner);
    Self { debug, inner }
  }

  #[must_use]
  pub fn downcast<T: AnyEvent>(&self) -> Option<T> {
    self.inner.downcast_ref().cloned()
  }
}

impl Debug for Event {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.debug)
  }
}

pub struct EventPublisher {
  tx: broadcast::Sender<Event>,
}

impl EventPublisher {
  #[must_use]
  pub fn new() -> Self {
    let (tx, _) = broadcast::channel(10);
    Self { tx }
  }

  #[must_use]
  pub fn subscribe(&self) -> broadcast::Receiver<Event> {
    self.tx.subscribe()
  }

  pub fn send(&self, event: impl AnyEvent) -> &Self {
    self._send(Event::new(event));
    self
  }

  fn _send(&self, event: Event) {
    log::debug!("Publish event {event:?}");
    if let Err(err) = self.tx.send(event) {
      log::error!("Can't publish event: {err}");
    }
  }

  /// Publish task local events
  pub fn flush(&self) {
    for event in event_bus::drain() {
      self._send(event);
    }
  }
}

impl Default for EventPublisher {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Debug, Clone)]
  struct Tested;

  #[tokio::test]
  async fn subscribe_any_use_case() {
    let event_publisher = EventPublisher::new();
    event_publisher.send(0);
    let mut rx = event_publisher.subscribe();
    let jh = tokio::spawn(async move {
      // should ignore int event because subscribed after
      let event1 = rx.recv().await.unwrap();
      let event2 = rx.recv().await.unwrap();
      event1
        .clone()
        .downcast::<Tested>()
        .ok_or("Expected None")
        .unwrap_err();
      event1.downcast::<String>().unwrap();
      event2.downcast::<Tested>().unwrap();
    });
    event_publisher.send("String event".to_string());
    event_publisher.send(Tested);
    jh.await.unwrap();
  }
}
