//! Types and functions for tasks on TaggyTime calendar.

use std::str::FromStr;

use crate::const_params::MAX_WORKLOAD;
use crate::time::date::Date;
use crate::time::timezone::ZoneOffset;
use crate::util_typs::percent::Percent;
use crate::time::*;

/// A wrapper around `u32`, which represents the number of minutes needed to
/// complete some task. Can only be from 0 to 60,000 (inclusive).
#[derive(Debug)]
pub struct Workload(u32);

impl Workload {
  /// Construct a `Workload` instance from some number of minutes. 
  /// Returns `Err` variant of out of bounds.
  pub fn from_num_min(num_min: u32) -> Result<Self, TimeError> {
    if num_min <= MAX_WORKLOAD {
      Ok(Workload(num_min))
    } else {
      Err(TimeError::WorkloadOverflowErr(num_min))
    }
  }

  /// Multiply a Workload instance by some percentage. Rounded to the nearest
  /// integer minute.
  pub fn multiply_percent(&self, p: Percent) -> Self {
    // will not overflow since such produce never exceeds 100 * 60_000.
    let workload_times_numerator = self.0 * (p.raw() as u32);

    let mut divided_by_denominator = workload_times_numerator / 100;

    // rounding up
    if workload_times_numerator % 100 >= 50 {
      divided_by_denominator += 1;
    }

    Workload(divided_by_denominator)
  }

  /// Returns the duration, in number of minutes, of such a workload.
  pub fn num_min(&self) -> u32 {
    self.0
  }
}

impl FromStr for Workload {
  type Err = TimeError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Workload::from_num_min(crate::time::parse_u32(s)?)
  }
}

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
#[derive(Debug)]
pub struct Todo {
  pub name: String,
  pub due: MinInstant,
  pub length: Workload,
  pub completion: Percent,
  cached_impact: Option<Percent>,
}

impl Todo {

  /// Constructs a new instance with zero completion.
  pub fn new(name: String, due: MinInstant, length: Workload) -> Self {
    Todo { name, due, length, completion: Percent::zero(), cached_impact: None }
  }

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

  pub fn from_str_triplet(
    name: &str, 
    due: &str, 
    load: &str, 
    default_tz: ZoneOffset
  ) -> Result<Self, TimeError> {
    Ok(Todo::new(
      name.to_string(), 
      MinInstant::from_date(&Date::parse_from_str(due, default_tz)?)?, 
      load.parse()?,
    ))
  }
}

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
