use std::mem;

use crate::ics_parser::ics_syntax::{FreqAndRRules, Vevent};
use crate::ics_parser::ICSProcessError;
use crate::time::date::Date;
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
      None => Ok(Pattern::Once),
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
  pub fn new(event_miv: MinInterval, patt: Pattern) -> Self {
    Self {
      event_miv,
      occurrence_count: OneOrMore::new(1).unwrap(),
      patt,
    }
  }

  /// Computes the next occurrence of the recurrence. If passes termination
  /// condition, returns `None`.
  pub fn next(self) -> Option<Self> {
    let tmr = self.event_miv.advance_unwrap(MIN_IN_DAY);
    let event_miv = match &self.patt {
      Pattern::Once => return None,
      Pattern::Many(dp, _, Term::Count(n)) => {
        if self.occurrence_count >= *n {
          return None;
        }
        tmr
          .advance_until_unwrap(dp, None)
          .expect("Unreachable: no until")
      }
      Pattern::Many(dp, _, Term::Until(term_mi)) => {
        tmr.advance_until_unwrap(dp, Some(*term_mi))?
      }
      Pattern::Many(dp, _, Term::Never) => tmr
        .advance_until_unwrap(dp, None)
        .expect("Unreachable: no until"),
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
    'a: for rec_miv in self {
      // skip the non-interacting min-intervals.
      if rec_miv.end <= miv.start {
        continue 'a;
      }
      if rec_miv.start >= miv.end {
        break 'a;
      }

      ret = ret
        .checked_add(rec_miv.overlap_duration(miv))
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
    Ok(Recurrence::new(value.miv, Pattern::try_from(value.repeat)?))
  }
}

/// An iterator for the `MinInterval` items in some recurrence.
pub struct RecIter {
  rec: Option<Recurrence>,
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

/// A struct that pairs the summary of some event with its `Recurrence`.
pub struct Event(pub String, pub Recurrence);

impl TryFrom<Vevent> for Event {
  type Error = ICSProcessError;

  fn try_from(value: Vevent) -> Result<Self, Self::Error> {
    Ok(Event(value.summary.clone(), Recurrence::try_from(value)?))
  }
}

impl std::fmt::Display for Event {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}\n{}\n", self.0, self.1)
  }
}

impl std::fmt::Display for Recurrence {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}\n{}", self.event_miv.as_date_string(), self.patt)
  }
}

impl std::fmt::Display for Pattern {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Pattern::Once => write!(f, "No repeat"),
      Pattern::Many(dp, iv, t) => {
        write!(f, "{:?}\nOccurs every {} times\n{}", dp, iv, t)
      }
    }
  }
}

impl std::fmt::Display for Term {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Term::Count(n) => write!(f, "Repeat {} times", n),
      Term::Until(mi) => write!(f, "Until {}", Date::from_min_instant(*mi)),
      Term::Never => Ok(()),
    }
  }
}

#[allow(dead_code, unused_imports)]
mod test {
  use super::*;

  #[test]
  fn rec_next() {
    let mi = MinInstant::from_raw_utc(27988182).unwrap();
    let mi2 = mi.advance(60).unwrap();
    let iv = MinInterval::new(mi, mi2);

    use crate::time::week::Weekday;
    let weeks = vec![Weekday::MO, Weekday::WE, Weekday::FR];
    let dp = DateProperty::from(weeks);

    let p = Pattern::Many(
      dp,
      OneOrMore::new(1).unwrap(),
      Term::Count(OneOrMore::new(12).unwrap()),
    );

    let mut r = Recurrence::new(iv, p);

    let mut last_string = String::new();
    loop {
      r = match r.next() {
        Some(rn) => rn,
        None => break,
      };
      last_string = r.event_miv.as_date_string();
    }

    assert_eq!(
      String::from(
        "2023/Apr/14 05:42, tz=+00:00 - 2023/Apr/14 06:42, tz=+00:00"
      ),
      last_string
    );
  }

  #[test]
  fn rec_iter() {
    let mi = MinInstant::from_raw_utc(27988182).unwrap();
    let mi2 = mi.advance(60).unwrap();
    let iv = MinInterval::new(mi, mi2);

    use crate::time::week::Weekday;
    let weeks = vec![Weekday::MO, Weekday::WE, Weekday::FR];
    let dp = DateProperty::from(weeks);

    let p = Pattern::Many(
      dp,
      OneOrMore::new(1).unwrap(),
      Term::Count(OneOrMore::new(12).unwrap()),
    );

    let r = Recurrence {
      event_miv: iv,
      occurrence_count: OneOrMore::new(1).unwrap(),
      patt: p,
    };

    let mut it = r.into_iter();

    let mut last_string = String::new();
    loop {
      let tmp = match it.next() {
        Some(rn) => rn,
        None => break,
      };
      last_string = tmp.as_date_string();
    }

    assert_eq!(
      String::from(
        "2023/Apr/14 05:42, tz=+00:00 - 2023/Apr/14 06:42, tz=+00:00"
      ),
      last_string
    );
  }

  #[test]
  fn rec_overlap() {
    let mi = MinInstant::from_raw_utc(28038182).unwrap(); // sunday
    let mi2 = mi.advance(MIN_IN_DAY * 5 - 720).unwrap(); // friday
    let miv = MinInterval::new(mi, mi2); // 2023/04/23 23:02 - 04/28 11:02

    let cls_start = MinInstant::from_raw_utc(27900600).unwrap();
    let cls_end = cls_start.advance(120).unwrap();
    let cls = MinInterval::new(cls_start, cls_end); // 2023/01/18 10:00-12:00
    let dp = {
      use crate::time::week::Weekday::*;
      DateProperty::from(vec![MO, WE, FR])
    };
    let p = Pattern::Many(dp, OneOrMore::new(1).unwrap(), Term::Never);
    let cls_rec = Recurrence::new(cls, p);

    assert_eq!(302, cls_rec.overlap(miv));
  }
}
