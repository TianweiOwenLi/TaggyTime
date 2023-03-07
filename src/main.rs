mod command_parser;
mod tasks_util;
mod time;
mod calendar;
mod percent;
mod ics_parser;
mod args;
mod error;

use std::io::BufRead;

use crate::args::*;

fn handle_command_vec(cmd: Vec<String>) -> Result<(), String> {
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
