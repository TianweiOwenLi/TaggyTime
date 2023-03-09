mod args;
mod calendar;
mod const_params;
mod ics_parser;
mod percent;
mod time;
mod util_typs;

use std::io::BufRead;

use time::timezone::ZoneOffset;

use crate::{args::*, time::Date};

/// Stores global variables for such interaction.
/// 
/// [todo] Implement load from file.
struct TaggyEnv {
  tz: ZoneOffset
}

/// Loads the interactive environment. 
fn load_env() -> Result<TaggyEnv, String> {
  Ok(TaggyEnv { tz: ZoneOffset::utc() })
}

/// Stores the interactive environment.
fn store_env() -> Result<(), String> {
  Ok(())
}

fn handle_command_vec(cmd: Vec<String>, tenv: &mut TaggyEnv) -> Result<(), String> {
  let cmd: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
  match cmd[..] {
    ["test", "lexer", ics_filename] => {
      match ics_parser::test_lexer(ics_filename) {
        Ok(()) => Ok(()),
        Err(e) => Err(e.to_string()),
      }
    }
    ["test", "parser", ics_filename] => {
      match ics_parser::test_parser(ics_filename) {
        Ok(()) => Ok(()),
        Err(e) => Err(e.to_string()),
      }
    }
    ["now"] => {
      let mut mi = time::MinInstant::now();
      mi.set_offset(tenv.tz);
      println!("The time now is: {}, offset={}", 
        Date::from_min_instant(mi),
        mi.get_offset(),
      );
      Ok(())
    }
    ["set", "tz", "-05:00"] => {
      tenv.tz = ZoneOffset::new(-300).unwrap();
      Ok(())
    }
    _ => Err("Invalid command".to_string()),
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
        eprintln!("[taggytime] Command error: {}", e)
      }
    }
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
