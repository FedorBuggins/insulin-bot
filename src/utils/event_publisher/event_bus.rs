//! Utils for domain events emitted in task local scope

use std::{future::Future, sync::Mutex};

use super::{AnyEvent, Event};

tokio::task_local! {
  static EVENTS: Mutex<Vec<Event>>;
}

pub async fn scope<F: Future>(f: F) -> F::Output {
  EVENTS.scope(Mutex::default(), f).await
}

pub fn push(event: impl AnyEvent) {
  let _ = EVENTS.try_with(|events| {
    events.lock().unwrap().push(Event::new(event));
  });
}

pub(super) fn drain() -> Vec<Event> {
  EVENTS
    .try_with(|events| events.lock().unwrap().drain(..).collect())
    .unwrap_or_default()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_no_effect_without_scope() {
    push(1);
    push(2);
    assert!(drain().is_empty());
  }

  #[tokio::test]
  async fn should_be_available_in_scope() {
    let events = scope(async {
      push(1u8);
      push(2u8);
      drain()
    })
    .await;
    let values: Vec<_> = events
      .into_iter()
      .filter_map(|event| event.downcast::<u8>())
      .collect();
    assert_eq!(vec![1, 2], values);
  }
}
