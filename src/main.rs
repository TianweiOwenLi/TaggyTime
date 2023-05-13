mod args;
mod calendar;
mod const_params;
mod ics_parser;
mod load_file;
mod time;
mod util_typs;
mod taggy_cmd;

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
struct TaggyEnv {
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
    ["now"] => {
      let mut mi = time::MinInstant::now();
      mi.adjust_to_zone(tenv.tz);
      println!("[taggytime] now is: {}", mi.as_date_string());
      Ok(NextInteraction::Prompt)
    }
    ["set", "tz", s] => {
      tenv.tz = s.parse()?;
      println!("[taggytime] timezone set to {}", tenv.tz);
      Ok(NextInteraction::Prompt)
    }
    ["load", filename, "as", newname] => {
      unimplemented!("Moved");
    }
    ["remove", name] => {
      if tenv.calendars.remove(name).is_none() {
        println!("[taggytime] There is no calendar `{}` to remove", name);
      }
      Ok(NextInteraction::Prompt)
    }
    ["add-todo", name, load, ..] => {
      unimplemented!("Moved")
    }
    ["set-progress", name, progress] => {
      match tenv.todolist.get_mut(name) {
        Some(task) => {
          let prog: Percent = progress.parse()?;
          task.set_progress(prog);
          println!("[taggytime] Progress set to {}", prog);
        }
        None => println!("[taggytime] Task `{}` does not exist", name),
      }
      Ok(NextInteraction::Prompt)
    }
    ["impact", name] => {
      let task_opt = tenv.todolist.get_ref(name);
      match task_opt {
        Some(task) => {
          println!("[taggytime] Impact = {}", tenv.calendars.impact(task))
        }
        None => println!("[taggytime] Task `{}` does not exist", name),
      }
      Ok(NextInteraction::Prompt)
    }
    ["truncate"] => {
      tenv.calendars.filter_events(|e| ! e.ended());
      Ok(NextInteraction::Prompt)
    }
    ["q"] | ["quit"] => Ok(NextInteraction::Quit),
    _ => Err(TimeError::InvalidCommand(format!("{:?}", cmd))),
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
