use std::{env, fmt::Display, path::Path};

use chrono::Local;
use tokio::{
  fs::OpenOptions,
  io::{self, AsyncWriteExt},
};

pub fn error(err: impl Display) {
  log::error!("{err}");
  tokio::spawn(alert(err.to_string()));
}

async fn alert(err: String) -> io::Result<()> {
  let date_time = Local::now()
    .to_rfc3339()
    .replace(|ch: char| !ch.is_ascii_digit(), "");
  let path = env::current_dir()?.join("errors").join(date_time);
  write_error_to_file(&path, &err).await?;
  Ok(())
}

async fn write_error_to_file(
  path: &Path,
  err: &str,
) -> io::Result<()> {
  let err = err.to_string() + "\n\n\n";
  let mut f = OpenOptions::new()
    .append(true)
    .create(true)
    .open(path)
    .await?;
  f.write_all(err.as_bytes()).await?;
  Ok(())
}
