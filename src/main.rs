mod args;
mod calendar;
mod const_params;
mod ics_parser;
mod load_file;
mod time;
mod util_typs;

use std::{io::BufRead, path::Path};

use serde::{Deserialize, Serialize};

use calendar::{
  cal_event::Event,
  task::{Task, Workload},
  NameMap,
};
use const_params::DBG;
use time::{timezone::ZoneOffset, MinInstant, TimeError};

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

/// Given some `.ics` file, loads it to some `TaggyEnv`. If an optional name is
/// provided, the loaded calendar will be renamed accordingly.
fn load_ics_to_tenv(
  tenv: &mut TaggyEnv,
  filename: &str,
  newname_opt: Option<&str>,
) {
  if tenv.calendars.contains(filename) {
    println!("[taggytime] Calendar `{}` already exists! ", filename);
  } else {
    let events = load_file::load_schedule_ics(filename, tenv.tz)
      .expect("[taggytime] Failed to .ics file");
    if DBG {
      for event in &events {
        println!("{}", event);
      }
    }
    tenv.calendars.force_insert(filename, events);

    match newname_opt {
      Some(newname) => {
        tenv
          .calendars
          .rename(filename, newname)
          .expect("Just inserted");
        println!(
          "[taggytime] Successfully loaded `{}` as `{}`",
          filename, newname
        );
      }
      None => println!("[taggytime] Successfully loaded `{}`", filename),
    }
  }
}

fn load_todo_to_tenv(tenv: &mut TaggyEnv, name: &str, todo: Task) {
  if tenv.todolist.contains(name) {
    println!("[taggytime] Task `{}` already exists! ", name);
  } else {
    if DBG {
      println!("{}", &todo);
    }
    tenv.todolist.force_insert(name, todo);
    println!("[taggytime] Successfully added task `{}`", name);
  }
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
    ["load", filename] => {
      if filename.ends_with(".ics") {
        load_ics_to_tenv(tenv, filename, None);
      } else {
        println!("[taggytime] Invalid file extension: {}", filename);
      }
      Ok(NextInteraction::Prompt)
    }
    ["load", filename, "as", newname] => {
      if filename.ends_with(".ics") {
        load_ics_to_tenv(tenv, filename, Some(newname));
      } else {
        println!("[taggytime] Invalid file extension: {}", filename);
      }
      Ok(NextInteraction::Prompt)
    }
    ["rename", old_name, new_name] => {
      tenv.calendars.rename(old_name, new_name)?;
      Ok(NextInteraction::Prompt)
    }
    ["remove", name] => {
      tenv.calendars.remove(name);
      Ok(NextInteraction::Prompt)
    }
    ["add-todo", name, load, ..] => {
      let load: Workload = load.parse()?;
      let due = MinInstant::parse_from_str(&cmd[3..], tenv.tz)?;
      let todo = Task::new(due, load);
      load_todo_to_tenv(tenv, name, todo);
      Ok(NextInteraction::Prompt)
    }
    ["set-progress", name, progress] => {
      let task_opt = tenv.todolist.get_mut(name);
      match task_opt {
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
    ["q"] | ["quit"] => {
      Ok(NextInteraction::Quit)
    }
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
    let v: Vec<String> =
      buf.split(' ').map(|s| s.trim().to_string()).collect();
    match handle_command_vec(v, tenv) {
      Ok(NextInteraction::Quit) => {
        break Ok(());
      }
      Ok(NextInteraction::Prompt) => {
        // do nothing
      }
      Err(e) => {
        eprintln!("[taggytime] Command error: {:?}", e);
      }
    }
  }
}

fn main() {
  let cli_info = parse_args().unwrap_or_else(|e| {
    eprintln!("Parse argument failed! \n{:?}", e);
    std::process::exit(1)
  });

  let mut tenv = load_env(&cli_info.taggyenv_path).unwrap_or_else(|e| {
    eprintln!("TaggyTime environment failed to load! \n{:?}", e);
    std::process::exit(1)
  });

  let run_result = match cli_info.mode {
    Mode::Interactive => {
      if let Err(e) = interactive_loop(&mut tenv) {
        eprintln!("[taggytime] Interactive mode error: {:?}", e);
      }
      store_env(&cli_info.taggyenv_path, &tenv)
    }
    Mode::Cli(v) => {
      handle_command_vec(v, &mut tenv).unwrap_or_else(|e| {
        eprintln!("Command execution failed! \n{:?}", e);
        std::process::exit(1)
      });
      store_env(&cli_info.taggyenv_path, &tenv)
    }
    Mode::Template => {
      store_empty_env(&cli_info.taggyenv_path)
    }
  };

  if let Err(e) = run_result {
    eprintln!("App encountered error: \n{:?}", e)
  }
}
