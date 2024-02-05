use std::{collections::HashSet, env, error::Error};

pub fn init() -> Result<(), Box<dyn Error>> {
  let env_filename =
    parse_arg(["--env", "-e"]).unwrap_or(".env".into());
  dotenv::from_filename(env_filename)?;
  Ok(())
}

fn parse_arg<I>(aliases: I) -> Option<String>
where
  I: IntoIterator,
  I::Item: Into<String>,
{
  let aliases: HashSet<String> =
    aliases.into_iter().map(<_>::into).collect();
  let mut args = env::args().skip(1);
  loop {
    if aliases.contains(args.next()?.as_str()) {
      return args.next();
    }
  }
}
