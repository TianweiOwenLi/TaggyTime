use std::mem;

use crate::const_params::MAX_WORKLOAD;
use crate::ics_parser::ics_syntax::{FreqAndRRules, Vevent};
use crate::ics_parser::ICSProcessError;
use crate::percent::Percent;
use crate::time::fact::MIN_IN_DAY;
use crate::time::{date::DateProperty, MinInstant, MinInterval};
use crate::util_typs::refinement::*;

pub type OneOrMore = LowerBoundI64<1>;

/// Occurrence skip interval, ie. happens every x (x >= 1) times.
pub type Interval = OneOrMore;


/// Recurrence pattern, ie. biweekly on TU, TH
#[derive(Clone)]
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
#[derive(Clone, Copy)]
pub enum Term {
  Count(OneOrMore),
  Until(MinInstant),
  Never,
}

/// Describes when shall some recurring events happen. This can correspond
/// to some mapping from `MinInstant` to `bool`, indicating precisely if a
/// recurring event is happening.
#[derive(Clone)]
pub struct Recurrence {
  /// Actual time interval of event, ie. 08:30 - 09:50
  event_miv: MinInterval,

  /// Indicates that `event_miv` is the nth occurrence. Shall be initialized as 1.
  occurrence_count: OneOrMore,

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
  pub fn next(self) -> Option<Self> {
    let tmr = self.event_miv.advance_unwrap(MIN_IN_DAY);
    let event_miv = match &self.patt {
      Pattern::Once => return None,
      Pattern::Many(dp, iv, Term::Count(n)) => {
        if self.occurrence_count >= *n { return None; }
        tmr.advance_until_unwrap(dp, None).expect("Unreachable: no until")
      }
      Pattern::Many(dp, iv, Term::Until(term_mi)) => {
        tmr.advance_until_unwrap(dp, Some(*term_mi))?
      }
      Pattern::Many(dp, iv, Term::Never) => {
        tmr.advance_until_unwrap(dp, None).expect("Unreachable: no until")
      }
    };
    Some(Recurrence {
      event_miv,
      occurrence_count: self.occurrence_count.increment_unwrap(),
      patt: self.patt,
    })
  }

  /// Computes the number of minutes overlapped with some `MinInterval`.
  pub fn overlap(self, miv: MinInterval) -> u32 {
    let mut ret: u32 = 0;
    for rec_miv in self {
      ret = ret.checked_add(rec_miv.overlap_duration(miv))
        .expect("Overflowed while computing recurrence and miv overlap");
    }
    ret
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


/// An iterator for the `MinInterval` items in some recurrence.
pub struct RecIter {
  rec: Option<Recurrence>
}

impl Iterator for RecIter {
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
  type IntoIter = RecIter;

  fn into_iter(self) -> Self::IntoIter {
    RecIter { rec: Some(self) }
  }
}

/// A wrapper around `u32`, which represents the number of minutes needed to
/// complete some task. Can only be from 0 to 60,000 (inclusive).
pub struct Workload(u32);

impl Workload {
  /// Construct a `Workload` instance from some number of minutes. 
  /// Returns `Err` variant of out of bounds.
  pub fn from_num_min(num_min: u32) -> Result<Self, String> {
    if num_min <= MAX_WORKLOAD {
      Ok(Workload(num_min))
    } else {
      Err("Workload cannot exceed 60,000 minutes".to_string())
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

    let r = Recurrence {
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
