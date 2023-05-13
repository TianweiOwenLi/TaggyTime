//! Handles taggytime commands.

use std::path::{PathBuf, Path};

use clap::Subcommand;

use crate::{time::TimeError, TaggyEnv, load_file, calendar::task::Task, util::path2string};

/// Given some `.ics` file, loads it to some `TaggyEnv`. If an optional name is
/// provided, the loaded calendar will be renamed accordingly.
fn load_ics_to_tenv<P: AsRef<Path>>(
  tenv: &mut TaggyEnv,
  path: P,
  name: &str,
) -> Result<(), TimeError> {
  let events = load_file::load_schedule_ics(&path, tenv.tz)?;
    tenv.calendars.unique_insert(name, events)?;
    println!("[taggytime] Loaded `{}` as `{}`", path2string(&path), name);
    Ok(())
}

fn load_todo_to_tenv(tenv: &mut TaggyEnv, name: &str, todo: Task) 
-> Result<(), TimeError>{
  tenv.todolist.unique_insert(name, todo)?;
  println!("[taggytime] Successfully added task `{}`", name);
  Ok(())
}

pub enum TaggyCmdError {
  TimeErr(TimeError),
}

impl From<TimeError> for TaggyCmdError {
  fn from(value: TimeError) -> Self {
    TaggyCmdError::TimeErr(value)
  }
}

#[derive(Subcommand)]
pub enum TaggyCmd {
  /// Loads some .ics calendar and gives it a name.
  CalLoad{
    /// Path to .ics file
    path: PathBuf,
    /// Preferred name of calendar
    name: String,
  },

  /// Removes some .ics calendar.
  CalRm{
    /// Name of calendar
    name: String,
  },

  /// Shows current time.
  Now,

  /// Shows current timezone.
  Tz,

  /// Sets Timezone.
  TzSet{
    /// Timezone string expression, i.e. -4:00 means EDT.
    tz_expr: String,
  },

  /// Adds new task.
  AddTask{
    /// Name of task.
    task: String,

    /// Workload of task in minutes.
    load: u32,

    /// Due date in string expression.
    duedate: String,

    /// Due time in string expression. 
    duetime: String,
  }
}
// let load: Workload = load.parse()?;
//       let due = MinInstant::parse_from_str(&cmd[3..], tenv.tz)?;
//       let todo = Task::new(due, load);
//       load_todo_to_tenv(tenv, name, todo)?;

impl TaggyCmd {
  pub fn handle(&self, tenv: &mut TaggyEnv) -> Result<(), TaggyCmdError> {
    use TaggyCmd::*;
    // use TaggyCmdError::*;
    match self {
      CalLoad { path, name } => {
        Ok(load_ics_to_tenv(tenv, path, name)?)
      }
    }
  }
}