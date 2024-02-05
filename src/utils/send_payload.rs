use std::future::Future;

use serde::{de::DeserializeOwned, Serialize};
use teloxide::{
  requests::{JsonRequest, Payload, ResponseResult},
  Bot,
};

pub trait SendPayload {
  type Output;
  fn send_by(
    self,
    bot: Bot,
  ) -> impl Future<Output = ResponseResult<Self::Output>> + Send;
}

impl<P> SendPayload for P
where
  P: Payload + Send + Serialize + 'static,
  P::Output: DeserializeOwned + Send,
{
  type Output = P::Output;
  async fn send_by(self, bot: Bot) -> ResponseResult<P::Output> {
    JsonRequest::new(bot, self).await
  }
}
