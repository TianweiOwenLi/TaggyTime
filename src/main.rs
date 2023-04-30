mod args;
mod calendar;
mod const_params;
mod ics_parser;
mod load_file;
mod time;
mod util_typs;

use std::io::BufRead;

use calendar::{Calendars, CalError, task::Todo};
use const_params::DBG;
use time::{timezone::ZoneOffset, TimeError};

use crate::{args::*, time::date::Date};

/// Stores global variables for such interaction.
///
/// [todo] Implement load from file.
struct TaggyEnv {
  tz: ZoneOffset,
  calendars: Calendars,
  prompt_stack: Vec<Prompt>,
  todolist: Vec<Todo>,
}

/// A user-promptable lambda.
struct Prompt {
  pub description: String,
  pub lambda: Box<dyn FnOnce()>
}

impl Prompt {
  /// Given a user response of yes or no, consumes the prompt; executes it if 
  /// user choosed yes.
  fn consume(self, usr_choice: bool) {
    if usr_choice { (self.lambda)() }
  }
}

/// Loads the interactive environment.
fn load_env() -> Result<TaggyEnv, String> {
  Ok(TaggyEnv {
    tz: ZoneOffset::utc(),
    calendars: Calendars::mk_empty(),
    prompt_stack: vec![],
    todolist: vec![],
  })
}

/// Stores the interactive environment.
fn store_env() -> Result<(), String> {
  Ok(())
}

fn load_ics_to_tenv(
  tenv: &mut TaggyEnv, 
  filename: &str, 
  newname_opt: Option<&str>
) {
  if tenv.calendars.contains(filename) {
    println!("[taggytime] Calendar {} already exists! ", filename);
  } else {
    let events = load_file::load_schedule_ics(filename, tenv.tz)
      .expect("[taggytime] Failed to .ics file");
    if DBG {
      for event in &events { println!("{}", event); }
    }
    tenv.calendars.force_insert(filename, events);

    match newname_opt {
      Some(newname) => {
        tenv.calendars.rename(filename, newname).expect("Just inserted");
        println!("[taggytime] Successfully loaded {} as {}", filename, newname);
      }
      None => println!("[taggytime] Successfully loaded {}", filename)
    }
  }
}

fn handle_command_vec(
  cmd: Vec<String>,
  tenv: &mut TaggyEnv,
) -> Result<(), TimeError> {
  let cmd: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();

  if let Some(head) = tenv.prompt_stack.pop() {
    println!("[taggytime] {} (y/n)", head.description);
    match cmd[..] {
      ["y"] => head.consume(true),
      ["n"] => head.consume(false),
      _ => println!("[taggytime] Please answer prompt with (y/n).")
    }
    return Ok(());
  }

  match cmd[..] {
    ["test", "lexer", ics_filename] => {
      match ics_parser::test_lexer(ics_filename) {
        Ok(()) => Ok(()),
        Err(e) => Err(TimeError::ICSErr(e)),
      }
    }
    ["test", "parser", ics_filename] => {
      match ics_parser::test_parser(ics_filename) {
        Ok(()) => Ok(()),
        Err(e) => Err(TimeError::ICSErr(e)),
      }
    }
    ["now"] => {
      let mut mi = time::MinInstant::now();
      mi.adjust_to_zone(tenv.tz);
      println!(
        "The time now is: {}, offset={}",
        Date::from_min_instant(mi),
        mi.offset,
      );
      Ok(())
    }
    ["set", "tz", s] => {
      tenv.tz = s.parse()?;
      println!("[taggytime] timezone set to {}", tenv.tz);
      Ok(())
    }
    ["load", filename] => {
      if filename.ends_with(".ics") {
        load_ics_to_tenv(tenv, filename, None);
      } else {
        println!("[taggytime] Invalid file extension: {}", filename);
      }
      Ok(())
    }
    ["load", filename, "as", newname] => {
      if filename.ends_with(".ics") {
        load_ics_to_tenv(tenv, filename, Some(newname));
      } else {
        println!("[taggytime] Invalid file extension: {}", filename);
      }
      Ok(())
    }
    ["rename", old_name, new_name] => {
      match tenv.calendars.rename(old_name, new_name) {
        Ok(()) => {}
        Err(CalError::RenameNonexist(_)) => {
          println!("[taggytime] Calendar `{}` does not exist", old_name);
        }
      }
      Ok(())
    }
    ["remove", name] => {
      tenv.calendars.remove(name);
      Ok(())
    }
    ["add-todo", name, due, load] => {
      let todo = Todo::from_str_triplet(name, due, load, tenv.tz)?;
      println!("[taggytime] added todo {}", tenv.tz);
      Ok(())
    }
    _ => Err(TimeError::InvalidCommand(format!("{:?}", cmd)))
  }
}

fn main() {
  let mode = parse_args().unwrap_or_else(|e| {
    eprintln!("Parse argument failed! \n{:?}", e);
    std::process::exit(1)
  });

  let mut tenv = load_env().unwrap_or_else(|e| {
    eprintln!("TaggyTime environment failed to load! \n{:?}", e);
    std::process::exit(1)
  });

  let run_result = match mode {
    Mode::Interactive => loop {
      let mut buf = String::new();
      let stdin_agent = std::io::stdin();

      // read line
      {
        let mut stdin_handle = stdin_agent.lock();
        if let Err(e) = stdin_handle.read_line(&mut buf) {
          break Err(format!("Failed to read line: {}", e));
        }
      }

      // interpret
      let v: Vec<String> =
        buf.split(' ').map(|s| s.trim().to_string()).collect();
      if let Err(e) = handle_command_vec(v, &mut tenv) {
        eprintln!("[taggytime] Command error: {:?}", e)
      }
    },
    Mode::Cli(v) => {
      handle_command_vec(v, &mut tenv).unwrap_or_else(|e| {
        eprintln!("Command execution failed! \n{:?}", e);
        std::process::exit(1)
      });
      store_env()
    }
  };

  if let Err(e) = run_result {
    eprintln!("App encountered error: \n{}", e)
  }
}
