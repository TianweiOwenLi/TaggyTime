mod command_parser;
mod tasks_util;
mod time;

mod calendar;

mod percent;

mod ics_parser;

mod args;

use std::io::BufRead;

use crate::args::*;

use tasks_util::Tasks;
use time::MinInstant;

use crate::time::Date;
// use crate::taggy_io::datapoint;

/// handles user-input command.
fn handle_user_command<'a>(cmd: &String) -> Result<(), &'a str> {
  // TODO fg/bg
  if cmd.starts_with("log ") {
    println!(
      "[taggytime] logged msg {:?}",
      Tasks::from_str(cmd[4..].to_string()).unwrap()
    );
    Ok(())
  } else if cmd.starts_with("top") {
    println!("[taggytime] accessed top");
    Ok(())
  } else if cmd.starts_with("add-todo") {
    println!("[taggytime] added todo with msg {}", &cmd[8..]);
    Ok(())
  } else if cmd.starts_with("remove-todo") {
    // TODO fuzzy remove
    println!("[taggytime] removed todo with msg {}", &cmd[11..]);
    Ok(())
  } else {
    Err("Command not found")
  }
}

fn handle_command_vec(cmd: Vec<String>) -> Result<(), String> {
  let cmd: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
  match cmd[..] {
    ["test", "lexer", ics_filename] => {
      ics_parser::test_lexer(ics_filename.to_string())
    }
    _ => Err("Invalid command".to_string())
  }
}

fn main() {
  let mode = parse_args().unwrap_or_else(|e| {
    eprintln!("Parse argument failed! \n{:?}", e);
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
      let v: Vec<String> = buf.split(' ').map(|s| s.trim().to_string()).collect();
      if let Err(e) = handle_command_vec(v) {
        eprintln!("[taggytime] Command error: {}", e)
      }
    },
    Mode::Cli(v) => handle_command_vec(v),
  };

  if let Err(e) = run_result {
    eprintln!("App encountered error: \n{}", e)
  }
}
