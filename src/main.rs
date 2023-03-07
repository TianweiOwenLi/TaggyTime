mod command_parser;
mod tasks_util;
mod time;

mod calendar;

mod percent;

mod ics_parser;

mod args;

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
  println!("{:?}", cmd);
  Ok(())
}

fn main() {
  
  let mode = parse_args().unwrap_or_else(|e| {
    eprintln!("Parse argument failed! \n{:?}", e);
    std::process::exit(1)
  });

  let run_result = match mode {
    Mode::Interactive => loop {
      let mut buf = String::new();
      break Err("Interactive mode is unimplemented! ".to_string())
    }
    Mode::Cli(v) => {
      handle_command_vec(v)
    }
    Mode::Test(s) => {
      println!("Testing {}", s);
      Ok(())
    }
  };

  if let Err(e) = run_result {
    eprintln!("App encountered error: \n{}", e)
  }
}
