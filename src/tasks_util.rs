use crate::command_parser::Command;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Tasks {
  // SPECIAL is totally not an allusion to Fallout
  Sleep,       // naps do count
  Projects,    // extracurriculars + learning
  Exercise,    // physical exercise
  Classwork,   // only the mandatory ones
  Interaction, // interactions with humans
  Activity,    // fun!
  Logistic,    // anything else + meta
}

// fn starts_ends_with()

impl Tasks {
  pub fn from_str(task_str: String) -> Option<Self> {
    let parsed = Command::parse(task_str);
    match parsed {
      Some(Command::Atomic(s)) => match s.as_str() {
        "Sleep" => Some(Self::Sleep),
        "Projects" => Some(Self::Projects),
        "Exercise" => Some(Self::Exercise),
        "Classwork" => Some(Self::Classwork),
        "Interaction" => Some(Self::Interaction),
        "Activity" => Some(Self::Activity),
        "Logistics" => Some(Self::Logistic),
        dne => {
          println!("[taggytime] command \'{}\' does not exist! ", dne);
          None
        }
      },
      // command parser failed to parse
      _ => None,
    }
  }
}

mod test {
  use super::*;

  #[test]
  fn test_sleep() {
    assert_eq!(Some(Tasks::Sleep), Tasks::from_str(String::from("Sleep")));
  }
}
