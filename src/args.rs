use std::env;


pub enum Mode {
  Interactive,
  Test(String),
  Cli(Vec<String>),
}

pub struct CommandLineError(pub String);

impl std::fmt::Debug for CommandLineError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Command Line Error: {}", self.0)
  }
}

pub fn parse_args() -> Result<Mode, CommandLineError> {
  let args: Vec<String> = env::args().collect();

  let n = args.len();

  if n < 2 {
    return Err(CommandLineError("Not enough arguments".to_string()));
  }

  let maybe_flag = args[1].as_str();

  match maybe_flag {
    "-i" => {
      if n > 2 {
        Err(CommandLineError(
          "Redundant argument after interaction mode".to_string()
        ))
      } else {
        Ok(Mode::Interactive)
      }
    }
    "-t" => {
      match args.get(2) {
        Some(s) => {
          if n > 3 {
            Err(CommandLineError(
              "Redundant argument after test mode".to_string()
            ))
          } else {
            Ok(Mode::Test(s.clone()))
          }
        }
        None => Err(CommandLineError("Please specify test mode".to_string()))
      }
    }
    _ => {
      Ok(Mode::Cli(args[1..].to_vec()))
    }
  }
  
}
