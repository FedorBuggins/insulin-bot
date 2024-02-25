use std::fmt;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum Error {
  Sqlx(sqlx::Error),
  Teloxide(teloxide::RequestError),
  Other(Box<dyn std::error::Error + Send + Sync>),
}

impl Error {
  pub fn is_network_problem(&self) -> bool {
    matches!(self, Self::Teloxide(teloxide::RequestError::Network(_)))
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Error::Sqlx(err) => err.fmt(f),
      Error::Teloxide(err) => err.fmt(f),
      Error::Other(err) => err.fmt(f),
    }
  }
}

impl From<sqlx::Error> for Error {
  fn from(v: sqlx::Error) -> Self {
    Self::Sqlx(v)
  }
}

impl From<teloxide::RequestError> for Error {
  fn from(v: teloxide::RequestError) -> Self {
    Self::Teloxide(v)
  }
}

impl From<&str> for Error {
  fn from(v: &str) -> Self {
    Self::Other(v.into())
  }
}

impl From<String> for Error {
  fn from(v: String) -> Self {
    Self::Other(v.into())
  }
}

/// Maps `error` to [`Error::Other`]
pub fn any<E>(error: E) -> Error
where
  E: std::error::Error + Send + Sync + 'static,
{
  Error::Other(Box::new(error))
}

impl std::error::Error for Error {}
