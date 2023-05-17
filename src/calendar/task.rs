//! Types and functions for tasks on TaggyTime calendar.

use std::str::FromStr;

use crate::time::fact::SEC_IN_MIN_U32;
use crate::time::time_parser::parse_u32;
use crate::time::*;
use crate::util_typs::percent::Percent;
use crate::{const_params::MAX_WORKLOAD, util_typs::percent::PercentError};

use serde::{Deserialize, Serialize};

/// A wrapper around `u32`, which represents the number of minutes needed to
/// complete some task. Can only be from 0 to 60,000 (inclusive).
#[derive(Debug, Serialize, Deserialize)]
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
    Workload::from_num_min(parse_u32(s)?)
  }
}

impl std::fmt::Display for Workload {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let (hr, min) = (self.0 / SEC_IN_MIN_U32, self.0 % SEC_IN_MIN_U32);
    write!(f, "{:02}:{:02}", hr, min)
  }
}

/// The impact of some task, which is either some percentage (measures the
/// percent of remaining time needed to complete such a task), or
/// ``Expired'', if the task is deemed impossible to complete.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExpirableImpact {
  Current(Percent),
  Expired,
}

impl From<f32> for ExpirableImpact {
  fn from(value: f32) -> Self {
    match Percent::try_from(value) {
      Ok(p) => ExpirableImpact::Current(p),
      Err(PercentError::PercentF32Overflow(_)) => ExpirableImpact::Expired,
      Err(e) => unreachable!("`{}` never raised by Percent::try_from", e),
    }
  }
}

impl std::cmp::PartialOrd for ExpirableImpact {
  /// Makes partial comparison between impacts, where ``Expired'' is treated
  /// as infinite percent.
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    use std::cmp::Ordering::*;
    use ExpirableImpact::*;
    match (self, other) {
      (Expired, Expired) => Some(Equal),
      (Expired, _) => Some(Greater),
      (_, Expired) => Some(Less),
      (Current(pl), Current(pr)) => pl.partial_cmp(pr),
    }
  }
}

impl std::fmt::Display for ExpirableImpact {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use ExpirableImpact::*;
    match self {
      Current(p) => write!(f, "{}", p),
      Expired => write!(f, "Expired"),
    }
  }
}

/// A struct that represents some task to be done.
///
/// This struct contains the following fields:
///
/// `due`: the due date of such a task, represented as a `Recurrence`.
///
/// `length`: number of minutes needed to complete such a task from scratch.
///
/// `completion`: the progress of such a task, in percentage.
///
/// `cached_impact`: the ratio of completion time, relative to available time
/// before deadline. Can only be updated with an external schedule. This shall
/// be cached, and only refreshed if needed.
///
/// [todo] Implement recurrences for todo
#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
  pub due: MinInstant,
  pub length: Workload,
  pub completion: Percent,
}

impl Task {
  /// Constructs a new instance with zero completion.
  pub fn new(due: MinInstant, length: Workload) -> Self {
    Task { due, length, completion: Percent(0) }
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
    self.completion =
      if tgt_progress.is_overflow() { Percent(100) } else { tgt_progress };
  }
}
