//! Handles taggytime commands.

use std::path::{Path, PathBuf};

use clap::Subcommand;

use crate::{
  calendar::task::{Task, Workload},
  load_file,
  time::{self, timezone::ZoneOffset, MinInstant, TimeError},
  util::path2string,
  util_typs::percent::{self, Percent},
  TaggyEnv,
};

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

fn load_todo_to_tenv(
  tenv: &mut TaggyEnv,
  name: &str,
  todo: Task,
) -> Result<(), TimeError> {
  tenv.todolist.unique_insert(name, todo)?;
  println!("[taggytime] Added task `{}`", name);
  Ok(())
}

/// Prints task impact for a given task, its name, and `TaggyEnv`.
pub fn prettyprint_task(
  name: &str,
  task: &Task,
  tz: ZoneOffset,
  impact: Percent,
) {
  println!(
    "{:<20} {:<20}  {:<8}      {:<10}        {:<10}",
    name,
    task.due.as_tz_date_string(tz),
    task.length,
    task.completion,
    impact
  )
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

impl From<percent::PercentError> for TaggyCmdError {
  fn from(value: percent::PercentError) -> Self {
    TaggyCmdError::TimeErr(TimeError::PercentErr(value))
  }
}

#[derive(Subcommand)]
pub enum TaggyCmd {
  /// Loads some .ics calendar and gives it a name.
  AddCal {
    /// Path to .ics file.
    path: PathBuf,
    /// Preferred name of calendar.
    name: String,
  },

  /// Removes some .ics calendar.
  RmCal {
    /// Name of calendar.
    name: String,
  },

  /// Shows current calendars
  Cals,

  /// Shows current events
  Events,

  /// Shows current time.
  Now,

  /// Shows current TaggyEnv timezone.
  Tz,

  /// Sets TaggyEnv timezone.
  SetTz {
    /// Timezone string expression, i.e. -4:00 means EDT.
    tz_expr: String,
  },

  /// Adds new task.
  AddTask {
    /// Name of task.
    task_name: String,

    /// Workload of task in minutes.
    load: u32,

    /// Due date in string expression.
    duedate: String,

    /// Due hour in string expression.
    duehour: String,

    /// Optional timezone specification. Defaults to TaggyEnv timezone.
    tz_opt: Option<String>,
  },

  /// Removes some task.
  RmTask {
    /// Name of task.
    taskname: String,
  },

  /// Sets the progress of a task.
  SetProgress {
    /// Name of task.
    task_name: String,

    /// Percentage as an integer.
    percent_raw: u16,
  },

  /// Shows the impact of all tasks.
  Impact,

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
      RmCal { name } => match tenv.calendars.remove(name) {
        Some(..) => println!("[taggytime] Removed calendar `{}`", name),
        None => println!("[taggytime] There is no calendar `{}`", name),
      },
      Truncate => {
        tenv.calendars.filter_events(|e| !e.ended());
      }
      Cals => {
        println!("[taggytime] Existing calendars: \n-------------------------");
        for (c, _) in tenv.calendars.iter() {
          println!("{}", c);
        }
      }
      Events => {
        println!("[taggytime] Existing events: \n-------------------------\n");
        for (_, v) in tenv.calendars.iter() {
          for e in v {
            println!("{}", e);
          }
        }
      }

      // time / timezone related operations
      Now => {
        let mi = time::MinInstant::now(tenv.tz);
        println!("[taggytime] now is: {}", mi.as_date_string());
      }
      Tz => {
        println!("[taggytime] timezone is {}", tenv.tz);
      }
      SetTz { tz_expr } => {
        tenv.tz = tz_expr.parse()?;
        println!("[taggytime] timezone set to {}", tenv.tz);
      }

      // task / progress related operations
      AddTask {
        task_name,
        load,
        duedate,
        duehour: duehr,
        tz_opt,
      } => {
        let mut due_parts: Vec<&str> = vec![duedate, duehr];
        if let Some(tz) = tz_opt {
          due_parts.push(tz);
        }

        let load: Workload = Workload::from_num_min(*load)?;
        let due = MinInstant::parse_from_str(&due_parts, tenv.tz)?;
        let todo = Task::new(due, load);
        load_todo_to_tenv(tenv, task_name, todo)?;
      }
      RmTask { taskname: task_name } => match tenv.todolist.remove(task_name) {
        Some(..) => println!("[taggytime] Removed task `{}`", task_name),
        None => println!("[taggytime] There is no task `{}`", task_name),
      },
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
      Impact => {
        let mut taskname_impact_pairs = Vec::<(&str, &Task, Percent)>::new();
        for (name, task) in tenv.todolist.iter() {
          taskname_impact_pairs.push((name, task, tenv.calendars.impact(task)));
        }

        taskname_impact_pairs.sort_by(|(n1, _, l1), (n2, _, l2)| {
          l2.partial_cmp(l1).unwrap_or(n2.cmp(n1))
        });

        println!(
          "\
Task Name            Due (tz={})       Workload   Progress  Impact
-----------------------------------------------------------------------------",
          tenv.tz
        );

        let mut percent_sum = Percent(0);
        for (name, task, impact) in &taskname_impact_pairs {
          prettyprint_task(name, task, tenv.tz, *impact);
          percent_sum = (percent_sum + *impact)?;
        }

        println!("\nSum of Impact: {}", percent_sum)
      }
    }
    Ok(())
  }
}
