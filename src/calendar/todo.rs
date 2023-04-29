use crate::calendar::cal_event::{Recurrence, Workload};
use crate::percent::Percent;
use crate::time::*;

/// A struct that represents some task to be done.
///
/// This struct contains the following fields:
///
/// `name`: a `String` representing the name of the task.
///
/// `due`: the due date of such a task, represented as a `Recurrence`.
///
/// `length`: number of minutes needed to complete such a task from scratch.
///
/// `completion`: the progress of such a task, in percentage.
///
/// `repeat`: the recurrence pattern of this task.
///
/// `cached_impact`: the ratio of completion time, relative to available time
/// before deadline. Can only be updated with an external schedule. This shall
/// be cached, and only refreshed if needed.
///
/// [todo] Implement recurrences for todo
pub struct Todo {
  name: String,
  due: MinInstant,
  length: Workload,
  completion: Percent,
  cached_impact: Option<Percent>,
}

pub struct TodoList {
  content: Vec<Todo>,
}

impl Todo {
  /// Computes the remaining workload of this `Todo` item, considering its
  /// `length` and `completion` fields.
  pub fn get_remaining_workload(&self) -> Workload {
    self.length.multiply_percent(
      self
        .completion
        .complement()
        .expect("progress complement cannot overflow"),
    )
  }

  /// Sets progress to `tgt_progress`, which is automatically constrained down
  /// to <= 100.
  pub fn set_progress(&mut self, tgt_progress: Percent) {
    self.completion = if tgt_progress <= Percent::one() {
      tgt_progress
    } else {
      Percent::new(100)
    };
  }
}

impl TodoList {}

#[allow(unused_imports)]
mod test {
  use super::*;

  fn nada() {
    let td = Todo {
      name: "Name".to_string(),
      due: MinInstant::now(),
      length: Workload::from_num_min(60).unwrap(),
      completion: Percent::new(0),
      cached_impact: None,
    };
  }
}
