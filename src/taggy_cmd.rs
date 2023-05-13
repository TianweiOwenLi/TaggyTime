//! Handles taggytime commands.

use std::path::{PathBuf, Path};

use clap::Subcommand;

use crate::{time::{TimeError, self}, TaggyEnv, load_file, calendar::task::Task, util::path2string};

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
  GetTz,

  /// Sets Timezone.
  SetTz{
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
  },

  /// Truncates already-ended events.
  Truncate,
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
      // calendar / events related operations
      CalLoad { path, name } => {
        load_ics_to_tenv(tenv, path, name)?;
      }
      CalRm { name } => {
        if tenv.calendars.remove(name).is_none() {
          println!("[taggytime] There is no calendar `{}` to remove", name);
        }
      }
      Truncate => {
        tenv.calendars.filter_events(|e| ! e.ended());
      }

      // time / timezone related operations
      Now => {
        let mut mi = time::MinInstant::now();
        mi.adjust_to_zone(tenv.tz);
        println!("[taggytime] now is: {}", mi.as_date_string());
      }
      GetTz => {
        println!("[taggytime] timezone is {}", tenv.tz);
      }
      SetTz { tz_expr } => {
        tenv.tz = tz_expr.parse()?;
        println!("[taggytime] timezone set to {}", tenv.tz);
      }
      
    }
    Ok(())
  }
}