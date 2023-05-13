//! Handles taggytime commands.

use std::path::{PathBuf, Path};

use clap::Subcommand;

use crate::{time::{TimeError, self, MinInstant}, TaggyEnv, load_file, calendar::task::{Task, Workload}, util::path2string, util_typs::percent::Percent};

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

/// Prints task impact for a given task, its name, and `TaggyEnv`.
pub fn print_task_impact(name: &str, task: &Task, tenv: &TaggyEnv) {
  println!("{:<25} {:<10}", name, tenv.calendars.impact(task))
}

#[derive(Debug)]
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
  AddCal{
    /// Path to .ics file.
    path: PathBuf,
    /// Preferred name of calendar.
    name: String,
  },

  /// Removes some .ics calendar.
  RmCal{
    /// Name of calendar.
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
    task_name: String,

    /// Workload of task in minutes.
    load: u32,

    /// Due in string expression.
    due: String,
  },

  /// Removes some task.
  RmTask{
    /// Name of task.
    task_name: String,
  },

  /// Sets the progress of a certain task.
  SetProgress{
    /// Name of task.
    task_name: String, 

    /// Percentage as an integer.
    percent_raw: u16,
  },

  /// Retrieves the impact of some or all tasks.
  Impact{
    /// Name of task.
    task_name_opt: Option<String>, 
  },

  /// Truncates already-ended events.
  Truncate,
}

impl TaggyCmd {
  pub fn handle(&self, tenv: &mut TaggyEnv) -> Result<(), TaggyCmdError> {
    use TaggyCmd::*;
    // use TaggyCmdError::*;
    match self {
      // calendar / events related operations
      AddCal { path, name } => {
        load_ics_to_tenv(tenv, path, name)?;
      }
      RmCal { name } => {
        match tenv.calendars.remove(name) {
          Some(..) => println!("[taggytime] Removed calendar `{}`", name),
          None => println!("[taggytime] There is no calendar `{}`", name),
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

      // task / progress related operations
      AddTask { task_name, load, due } => {
        println!("{}", due);

        let due_parts: Vec<&str> = due.split(' ').map(|s| s.trim()).collect();

        let load: Workload = Workload::from_num_min(*load)?;
        let due = MinInstant::parse_from_str(&due_parts, tenv.tz)?;
        let todo = Task::new(due, load);
        load_todo_to_tenv(tenv, task_name, todo)?;
      }
      RmTask { task_name } => {
        match tenv.todolist.remove(task_name) {
          Some(..) => println!("[taggytime] Removed task `{}`", task_name),
          None => println!("[taggytime] There is no task `{}`", task_name),
        }
      }
      SetProgress { task_name, percent_raw } => {
        match tenv.todolist.get_mut(task_name) {
          Some(task) => {
            let prog: Percent = Percent(*percent_raw);
            task.set_progress(prog);
            println!("[taggytime] Progress set to {}", prog);
          }
          None => println!("[taggytime] Task `{}` does not exist", task_name),
        }
      }
      Impact { task_name_opt: None } => {
        let mut taskname_impact_pairs = Vec::<(String, Percent)>::new();
        for (name, task) in tenv.todolist.iter() {     
          taskname_impact_pairs.push(
            (name.clone(), tenv.calendars.impact(task))
          );
        }

        taskname_impact_pairs.sort_by(|(n1, l1), (n2, l2)| {
          l2.partial_cmp(l1).unwrap_or(n2.cmp(n1))
        });

        for (name, load) in &taskname_impact_pairs {
          println!("{:<25} {:<10}", name, load);
        }
      }
      Impact { task_name_opt: Some(task_name) } => {
        match tenv.todolist.get_ref(task_name) {
          Some(task) => {
            print_task_impact(task_name, task, tenv);
          }
          None => println!("[taggytime] Task `{}` does not exist", task_name),
        }
      }
      
    }
    Ok(())
  }
}