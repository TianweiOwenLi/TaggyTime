use std::collections::BTreeSet;
use std::mem;

use datetime::Month;

use crate::ics_parser::ics_syntax::{RRuleToks, FreqAndRRules, Freq, Vevent};
use crate::ics_parser::ICSProcessError;
use crate::ics_parser::lexer::Token;
use crate::percent::Percent;
use crate::time::date::Date;
use crate::time::fact::MIN_IN_DAY;
use crate::time::TimeError;
use crate::time::week::Weekday;
use crate::time::{date::DateProperty, MinInstant, MinInterval};
use crate::util_typs::refinement::*;

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

pub type Minutes = RangedI64<0, 59>;
pub type Hours = RangedI64<0, 23>;

// pub type OrdWkDay = (OrdSign, WeekDay);
pub type OrdMoDay = (OrdSign, RangedI64<1, 31>);
pub type OrdYrDay = (OrdSign, RangedI64<1, 366>);
pub type OrdWkNum = (OrdSign, RangedI64<1, 53>);

pub type ByMinLst = BTreeSet<Minutes>;
pub type ByHrLst = BTreeSet<Hours>;
// pub type ByWkDayLst = BTreeSet<OrdWkDay>;
pub type ByMoDayLst = BTreeSet<OrdMoDay>;
pub type ByYrDayLst = BTreeSet<OrdYrDay>;
pub type ByWkNumLst = BTreeSet<OrdWkNum>;
pub type ByMonthLst = BTreeSet<Month>;

pub type OneOrMore = LowerBoundI64<1>;
pub type SetPos = Option<OneOrMore>;
pub type Interval = OneOrMore;
// pub type WeekStart = Option<WeekDay>;

pub enum Pattern {
  Once,
  Many(DateProperty, Interval, Term),
}

impl TryFrom<Option<FreqAndRRules>> for Pattern {
  type Error = ICSProcessError;
  fn try_from(value: Option<FreqAndRRules>) -> Result<Self, Self::Error> {
    match value {
      Some(frq) => {
        let dp = DateProperty::from(frq.content);
        let itv = OneOrMore::try_new(frq.interval)?;
        let term = match (frq.count, frq.until) {
          (None, None) => Term::Never,
          (None, Some(mi)) => Term::Until(mi),
          (Some(c), None) => Term::Count(OneOrMore::try_new(c)?),
          (Some(c), Some(mi)) => {
            return Err(ICSProcessError::UntilAndCountBothAppear(c, mi))
          }
        };

        Ok(Pattern::Many(dp, itv, term))
      }
      None => Ok(Pattern::Once)
    }
  }
}

/// Recurrence event termination condition, which is either a number of
/// occurrences, a "finished" time instance, or never.
pub enum Term {
  Count(OneOrMore),
  Until(MinInstant),
  Never,
}

/// Describes when shall some recurring events happen. This can correspond
/// to some mapping from `MinInstant` to `bool`, indicating precisely if a
/// recurring event is happening.
pub struct Recurrence {
  /// Actual time interval of event, ie. 08:30 - 09:50
  event_miv: MinInterval,

  /// Indicates that `event_miv` is the nth occurrence. Shall be initialized as 1.
  occurrence_count: OneOrMore,

  /// Recurrence pattern, ie. weekly on TU, TH
  patt: Pattern,
}

impl Recurrence {
  pub fn once(mi: MinInterval) -> Self {
    Self {
      event_miv: mi,
      occurrence_count: OneOrMore::new(1).unwrap(),
      patt: Pattern::Once,
    }
  }

  /// Computes the next occurrence of the recurrence. If passes termination
  /// condition, returns `None`.
  ///
  /// [todo] Advancement is at least one day.
  pub fn next(self) -> Option<Self> {
    match &self.patt {
      Pattern::Once => None,
      Pattern::Many(dp, iv, Term::Count(n)) => {
        if self.occurrence_count >= *n {
          None
        } else {
          let new_miv = self
            .event_miv
            .advance_unwrap(MIN_IN_DAY)
            .advance_until_unwrap(dp, None);
          Some(Recurrence {
            event_miv: new_miv
              .expect("Unreachable since term is count variant"),
            occurrence_count: self.occurrence_count.increment_unwrap(),
            patt: self.patt,
          })
        }
      }
      Pattern::Many(dp, iv, Term::Until(term_mi)) => {
        let new_miv_opt = self
          .event_miv
          .advance_unwrap(MIN_IN_DAY)
          .advance_until_unwrap(dp, Some(*term_mi));
        match new_miv_opt {
          Some(new_miv) => Some(Recurrence {
            event_miv: new_miv,
            occurrence_count: self.occurrence_count.increment_unwrap(),
            patt: self.patt,
          }),
          None => None,
        }
      }
      Pattern::Many(dp, iv, Term::Never) => {
        let new_miv = self
          .event_miv
          .advance_unwrap(MIN_IN_DAY)
          .advance_until_unwrap(dp, None);
        Some(Recurrence {
          event_miv: new_miv.expect("Unreachable since term is count variant"),
          occurrence_count: self.occurrence_count.increment_unwrap(),
          patt: self.patt,
        })
      }
    }
  }
}

impl TryFrom<Vevent> for Recurrence {
  type Error = ICSProcessError;

  /// Converts a parsed vector of rrules into a `Recurrence` instance. 
  /// 
  /// [warning] Only weekly - by weekday is implemented. 
  fn try_from(value: Vevent) -> Result<Self, Self::Error> {
    let patt = Pattern::try_from(value.repeat)?;
    Ok(Recurrence {
      event_miv: value.miv,
      occurrence_count: OneOrMore::new(1).unwrap(),
      patt
    })
  }
}

pub struct Iter {
  rec: Option<Recurrence>
}

impl Iterator for Iter {
  type Item = MinInterval;

  fn next(&mut self) -> Option<Self::Item> {
    // This is full of acrobatics......
    let old_rec = mem::replace(&mut self.rec, None);
    let ret = old_rec.as_ref()?.event_miv;
    self.rec = old_rec?.next();

    Some(ret)
  }
}

impl IntoIterator for Recurrence {
  type Item = MinInterval;
  type IntoIter = Iter;

  fn into_iter(self) -> Self::IntoIter {
    Iter { rec: Some(self) }
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


#[allow(dead_code, unused_imports)]
mod test {
  use super::*;

  #[test]
  fn rec_next() {
    let mi = MinInstant::from_raw(27988182).unwrap();
    let mi2 = mi.advance(60).unwrap();
    let iv = MinInterval::new(mi, mi2);

    use crate::time::week::Weekday;
    let weeks = vec![Weekday::MO, Weekday::WE, Weekday::FR];
    let dp = DateProperty::from(weeks);

    let p = Pattern::Many(dp, OneOrMore::new(1).unwrap(), Term::Count(OneOrMore::new(12).unwrap()));

    let mut r = Recurrence {
      event_miv: iv,
      occurrence_count: OneOrMore::new(1).unwrap(),
      patt: p
    };

    let mut last_string = String::new();
    loop {
      r = match r.next() {
        Some(rn) => rn,
        None => break,
      };
      last_string = r.event_miv.as_date_string();
    };

    assert_eq!(
      String::from("2023/Apr/14 05:42 - 2023/Apr/14 06:42"), 
      last_string
    );
  }

  #[test]
  fn rec_iter() {
    let mi = MinInstant::from_raw(27988182).unwrap();
    let mi2 = mi.advance(60).unwrap();
    let iv = MinInterval::new(mi, mi2);

    use crate::time::week::Weekday;
    let weeks = vec![Weekday::MO, Weekday::WE, Weekday::FR];
    let dp = DateProperty::from(weeks);

    let p = Pattern::Many(dp, OneOrMore::new(1).unwrap(), Term::Count(OneOrMore::new(12).unwrap()));

    let mut r = Recurrence {
      event_miv: iv,
      occurrence_count: OneOrMore::new(1).unwrap(),
      patt: p
    };

    let mut it = r.into_iter();

    let mut last_string = String::new();
    loop {
      let tmp = match it.next() {
        Some(rn) => rn,
        None => break,
      };
      last_string = tmp.as_date_string();
    };

    assert_eq!(
      String::from("2023/Apr/14 05:42 - 2023/Apr/14 06:42"), 
      last_string
    );

  }
}
