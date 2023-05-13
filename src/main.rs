mod args;
mod calendar;
mod const_params;
mod ics_parser;
mod load_file;
mod time;
mod util_typs;
mod taggy_cmd;
mod util;

use std::{io::BufRead, path::Path};

use clap::Parser;
use serde::{Deserialize, Serialize};

use calendar::{cal_event::Event, task::Task, NameMap};
use time::{timezone::ZoneOffset, TimeError};

use crate::{args::*, util_typs::percent::Percent};

enum NextInteraction {
  Prompt,
  Quit,
}

/// Stores global variables for such interaction.
///
/// [todo] Implement load from file.
#[derive(Serialize, Deserialize)]
pub struct TaggyEnv {
  tz: ZoneOffset,
  calendars: NameMap<Vec<Event>>,
  todolist: NameMap<Task>,
}

impl TaggyEnv {
  fn new() -> Self {
    TaggyEnv {
      tz: ZoneOffset::utc(),
      calendars: NameMap::mk_empty(),
      todolist: NameMap::mk_empty(),
    }
  }
}

/// Loads the interactive environment.
fn load_env<T: AsRef<Path>>(path: T) -> Result<TaggyEnv, TimeError> {
  let content = std::fs::read_to_string(path)?;
  Ok(serde_json::from_str(&content)?)
}

/// Stores the interactive environment.
fn store_env<T: AsRef<Path>>(path: T, tenv: &TaggyEnv) -> Result<(), TimeError> {
  let s = serde_json::to_string(tenv)?;
  Ok(std::fs::write(path, s)?)
}

/// Creates a new `json` file for a default (i.e. empty) environment.
fn store_empty_env<T: AsRef<Path>>(path: T) -> Result<(), TimeError> {
  store_env(path, &TaggyEnv::new())
}

fn handle_command_vec(
  cmd: Vec<String>,
  tenv: &mut TaggyEnv,
) -> Result<NextInteraction, TimeError> {
  let cmd: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();

  match cmd[..] {
    ["test", "lexer", ics_filename] => {
      ics_parser::test_lexer(ics_filename)?;
      Ok(NextInteraction::Prompt)
    }
    ["test", "parser", ics_filename] => {
      ics_parser::test_parser(ics_filename)?;
      Ok(NextInteraction::Prompt)
    }
    _ => panic!("bad command")
  }
}

fn interactive_loop(tenv: &mut TaggyEnv) -> Result<(), TimeError> {
  loop {
    let mut buf = String::new();
    let stdin_agent = std::io::stdin();

    // read line
    {
      let mut stdin_handle = stdin_agent.lock();
      stdin_handle.read_line(&mut buf)?;
    }

    // interpret
    let v = buf.split(' ').map(|s| s.trim().to_string()).collect();
    match handle_command_vec(v, tenv) {
      Ok(NextInteraction::Quit) => break Ok(()),
      Ok(NextInteraction::Prompt) => (),
      Err(e) => eprintln!("[taggytime] Command error: {:?}", e),
    }
  }
}

fn main() {
  let cli_info = CliInfo::parse();

  let mut tenv = load_env(&cli_info.envpath).unwrap_or_else(|e| {
    eprintln!("TaggyTime environment failed to load! \n{:?}", e);
    std::process::exit(1)
  });

  let run_result = match cli_info.mode {
    Mode::Interactive => {
      if let Err(e) = interactive_loop(&mut tenv) {
        eprintln!("[taggytime] Interactive mode error: {:?}", e);
      }
      store_env(&cli_info.envpath, &tenv)
    }
    Mode::Template => {
      store_empty_env(&cli_info.envpath)
    }
  };

  if let Err(e) = run_result {
    eprintln!("App encountered error: \n{:?}", e)
  }
}
