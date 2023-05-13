use std::env;

pub enum Mode {
  Interactive,
  Template,
  Cli(Vec<String>),
}

/// Stores information parsed from commandline args.
pub struct CliInfo {
  pub taggyenv_path: String,
  pub mode: Mode,
}

pub struct CommandLineError(pub String);

impl std::fmt::Debug for CommandLineError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Command Line Error: {}", self.0)
  }
}

pub fn parse_args() -> Result<CliInfo, CommandLineError> {
  let args: Vec<String> = env::args().collect();

  let n = args.len();

  if n < 3 {
    return Err(CommandLineError("Not enough arguments".to_string()));
  }

  let flag = args[1].as_str();
  let taggyenv_path = args[2].to_string();

  match flag {
    // Interactive mode
    "-i" => {
      if n > 3 {
        Err(CommandLineError(
          "Redundant argument after interaction mode".to_string(),
        ))
      } else {
        Ok(CliInfo { 
          taggyenv_path,
          mode: Mode::Interactive,
        })
      }
    }

    // Cli mode
    "-c" => Ok(CliInfo { 
      taggyenv_path,
      mode: Mode::Cli(args[1..].to_vec()) 
    }),

    // Create template
    "-t" => Ok(CliInfo { taggyenv_path, mode: Mode::Template }),

    // Bad flag
    _ => panic!("Bad flag")
  }
}
