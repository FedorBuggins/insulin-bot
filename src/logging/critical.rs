use std::{
  env,
  fmt::Display,
  path::Path,
  process::Command,
  sync::atomic::{AtomicU16, Ordering},
};

use chrono::Local;
use tokio::{
  fs::OpenOptions,
  io::{self, AsyncWriteExt},
};

static ID_COUNTER: AtomicU16 = AtomicU16::new(42);

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
  show_notification(&path, &err)?;
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

fn show_notification(path: &Path, err: &str) -> io::Result<()> {
  let id = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
  let archive_dir = path
    .parent()
    .unwrap_or(path)
    .join("archive")
    .to_string_lossy()
    .to_string();
  let logo_path = env::current_dir()?
    .join("bum.jpg")
    .to_string_lossy()
    .to_string();
  let path = path.to_string_lossy();
  Command::new("termux-notification")
    .args(["--id", &id.to_string()])
    .args(["--title", "Major Elephant Bot"])
    .args(["--content", err])
    .args(["--icon", "error"])
    .args(["--image-path", &logo_path])
    .args(["--button1", "Archive"])
    .args([
      "--button1-action",
      &format!(
        r#"
          mv {path} {archive_dir};
          termux-notification-remove {id}
        "#
      ),
    ])
    .args(["--button2", "Open"])
    .args(["--button2-action", &format!("termux-open {path}")])
    .spawn()?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn increment_id_counter_in_parallel() {
    ID_COUNTER.store(0, Ordering::Relaxed);
    let jh1 = tokio::spawn(async {
      ID_COUNTER.fetch_add(1, Ordering::Relaxed)
    });
    let jh2 = tokio::spawn(async {
      ID_COUNTER.fetch_add(1, Ordering::Relaxed)
    });
    jh1.await.unwrap();
    jh2.await.unwrap();
    assert_eq!(2, ID_COUNTER.load(Ordering::Relaxed));
  }
}
