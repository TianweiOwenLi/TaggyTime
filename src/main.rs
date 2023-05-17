mod args;
mod calendar;
mod const_params;
mod ics_parser;
mod load_file;
mod taggy_cmd;
mod time;
mod util;
mod util_typs;

use clap::Parser;
use const_params::TAGGYENV_PATH;
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
fn load_env() -> Result<TaggyEnv, TimeError> {
  let content = std::fs::read_to_string(TAGGYENV_PATH)?;
  Ok(serde_json::from_str(&content)?)
}

/// Stores the interactive environment.
fn store_env(tenv: &TaggyEnv) -> Result<(), TimeError> {
  let s = serde_json::to_string(tenv)?;
  Ok(std::fs::write(TAGGYENV_PATH, s)?)
}

fn main() {
  let cli_info = CliInfo::parse();

  let mut tenv = load_env().unwrap_or_else(|e| {
    eprintln!("App failed to load taggyenv: \n{:?}", e);
    std::process::exit(1)
  });

  if cli_info.interactive {
    todo!()
  } else {
    if let Err(e) = cli_info.cmd.handle(&mut tenv) {
      eprintln!("App encountered error: \n{:?}", e)
    }
    if let Err(e) = store_env(&tenv) {
      eprintln!("App failed to save taggyenv: \n{:?}", e)
    }
  }
}
