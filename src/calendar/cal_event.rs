use std::collections::BTreeSet;

use datetime::Month;

use crate::ics_parser::ICSProcessError;
use crate::ics_parser::ics_syntax::RRuleToks;
use crate::ics_parser::lexer::Token;
use crate::percent::Percent;
use crate::time::{MinInstant, MinInterval, date::DateProperty};
use crate::util_typs::refinement::*;

use crate::time::week::Weekday;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum RecurRules {
  ByMon,
  ByWkNo,
  ByYrDay,
  ByMoDay,
  ByWkDay,
  ByHr,
  BySetPos,
}

pub enum OrdSign {
  P,
  M,
}

pub type Minutes = RangedI64::<0, 59>;
pub type Hours = RangedI64::<0, 23>;

// pub type OrdWkDay = (OrdSign, WeekDay);
pub type OrdMoDay = (OrdSign, RangedI64::<1, 31>);
pub type OrdYrDay = (OrdSign, RangedI64::<1, 366>);
pub type OrdWkNum = (OrdSign, RangedI64::<1, 53>);

pub type ByMinLst = BTreeSet<Minutes>;
pub type ByHrLst = BTreeSet<Hours>;
// pub type ByWkDayLst = BTreeSet<OrdWkDay>;
pub type ByMoDayLst = BTreeSet<OrdMoDay>;
pub type ByYrDayLst = BTreeSet<OrdYrDay>;
pub type ByWkNumLst = BTreeSet<OrdWkNum>;
pub type ByMonthLst = BTreeSet<Month>;

pub type OneOrMore = LowerBoundI64<1>;
pub type SetPos = Option<OneOrMore>;
pub type Interval = Option<OneOrMore>;
// pub type WeekStart = Option<WeekDay>;

/// Repetition by day, week, month, year, etc.
// pub enum Repeat {
//   Daily(ByMonthLst, ByMoDayLst, ByWkDayLst, ByHrLst, SetPos),
//   Weekly(ByMonthLst, ByWkDayLst, ByHrLst, SetPos),
//   Monthly(ByMonthLst, ByMoDayLst, ByWkDayLst, ByHrLst, SetPos),
//   Yearly(ByMonthLst, ByWkNumLst, ByYrDayLst, ByMoDayLst, ByWkDayLst, ByHrLst, SetPos),
// }

pub struct Repeat {
  weekday: DateProperty
}

pub enum Pattern {
  Once,
  Many(Repeat, Interval, Term)
}

/// Recurrence event termination condition, which is either a number of 
/// occurrences, a "finished" time instance, or never. 
pub enum Term {
  Count(usize),
  Until(MinInstant),
  Never,
}

/// Describes when shall some recurring events happen. This can correspond 
/// to some mapping from `MinInstant` to `bool`, indicating precisely if a 
/// recurring event is happening.
pub struct Recurrence {
  /// Actual time interval of event, ie. 08:30 - 09:50
  event: MinInterval,

  /// Recurrence pattern, ie. weekly on TU, TH
  patt: Pattern,
}

impl Recurrence {
  pub fn once(mi: MinInterval) -> Self {
    Self { event: mi, patt: Pattern::Once }
  }
}

impl TryFrom<Vec<RRuleToks>> for Recurrence {
  type Error = ICSProcessError;
  fn try_from(value: Vec<RRuleToks>) -> Result<Self, Self::Error> {
    unimplemented!()
  }
}

/// A wrapper around u32, which represents the number of minutes needed to
/// complete some task. Such a u32 can only be from 0 to 60,000 (inclusive)
/// to prevent u32 multiplication overflow.
///
/// # Examples
/// ```
/// let w1: Workload = Workload::from_num_min(16);
/// let w1: Workload = Workload::from_num_min(15);
///
/// let p = Percent::from_u8(63);
///
/// let d1 = w1.get_duration(); // = 16
/// let d2 = w2
///   .multiply_percent(p)
///   .get_duration(); // 25 * (63%) = 15.75, which rounds to 16.
///
/// assert_eq!(d1, d2);
/// ```
pub struct Workload(u32);

impl Workload {
  /// Construct a `Workload` instance from some `u32`, which represents the
  /// number of minutes of such a workload. Only values from 0 to 60,000
  /// (inclusive) are allowed, in order to prevent u32 multiplication overflow.
  pub fn from_num_min(num_min: u32) -> Result<Self, String> {
    if num_min <= 60_000 {
      Ok(Workload(num_min))
    } else {
      Err("Workload is too high: cannot exceed 60,000 minutes".to_string())
    }
  }

  /// Multiply a Workload instance by some percentage. Rounded to the nearest
  /// integer minute.
  ///
  /// # Example
  /// ```
  /// assert_eq!(
  ///   31,
  ///   Workload(60).multiply_percent(Percent::from_u8(51))
  /// );
  /// ```
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
  pub fn get_duration(&self) -> u32 {
    self.0
  }
}

impl std::fmt::Display for Recurrence {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    unimplemented!()
  }
}
