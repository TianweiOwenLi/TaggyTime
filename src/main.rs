mod args;
mod calendar;
mod const_params;
mod ics_parser;
mod load_file;
mod taggy_cmd;
mod time;
mod util;
mod util_typs;

use std::path::Path;

use clap::Parser;
use const_params::TAGGYENV_RELATIVE_PATH;
use serde::{Deserialize, Serialize};

use calendar::{cal_event::Event, task::Task, NameMap};
use time::{timezone::ZoneOffset, TimeError};

use crate::args::*;

/// Taggytime environment, including calendar schedules
#[derive(Serialize, Deserialize)]
pub struct TaggyEnv {
  tz: ZoneOffset,
  calendars: NameMap<Vec<Event>>,
  todolist: NameMap<Task>,
}

/// Loads the interactive environment.
fn load_env<P: AsRef<Path>>(path: P) -> Result<TaggyEnv, TimeError> {
  let content = std::fs::read_to_string(path)?;
  Ok(serde_json::from_str(&content)?)
}

/// Stores the interactive environment.
fn store_env<P: AsRef<Path>>(
  path: P,
  tenv: &TaggyEnv,
) -> Result<(), TimeError> {
  let s = serde_json::to_string(tenv)?;
  Ok(std::fs::write(path, s)?)
}

fn main() {
  let mut tenv_abs_path =
    home::home_dir().expect("Cannot find home directory! ");
  tenv_abs_path.push(TAGGYENV_RELATIVE_PATH);

  let cli_info = CliInfo::parse();

  let mut tenv = load_env(&tenv_abs_path).unwrap_or_else(|e| {
    eprintln!("App failed to load taggyenv: \n{:?}", e);
    std::process::exit(1)
  });

  if cli_info.interactive {
    todo!()
  } else {
    if let Err(e) = cli_info.cmd.handle(&mut tenv) {
      eprintln!("App encountered error: \n{:?}", e)
    }
    if let Err(e) = store_env(&tenv_abs_path, &tenv) {
      eprintln!("App failed to save taggyenv: \n{:?}", e)
    }
  }
}
