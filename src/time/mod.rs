use core::panic;
use datetime::Instant;

mod year;
use year::{UnixYear, Year, YearError};

mod month;
use month::Month;

mod fact;

pub mod timezone;

use crate::calendar::cal_event::Workload;

use self::{fact::*, timezone::ZoneOffset, year::CeYear};

const SEC_IN_MIN: i64 = 60;

// these bounds prevent overflow during timezone adjustments.
const MINUTE_UPPERBOUND: i64 = u32::MAX as i64 - timezone::UTC_UB as i64;
const MINUTE_LOWERBOUND: i64 = u32::MIN as i64 - timezone::UTC_LB as i64;

// ---------------------------------- Utils -----------------------------------

/// Safely sums up an array of `u32`, returns `None` if overflows.
pub fn u32_safe_sum(numbers: &[u32]) -> Option<u32> {
  let mut ret: u32 = 0;
  for n in numbers {
    ret = ret.checked_add(*n)?;
  }
  Some(ret)
}

// ---------------------------------- Impls -----------------------------------

/// minutes since Unix Epoch. This can be casted to a different timezone
/// by incrementing both raw and offset at the same time, without changing
/// the actual time instant being represented.
#[derive(Debug, Clone, Copy)]
pub struct MinInstant {
  raw: u32,
  offset: ZoneOffset,
}

impl PartialEq for MinInstant {

  /// Tests whether two `MinInstant` equals. 
  /// 
  /// [todo] Improve efficiency.
  fn eq(&self, other: &Self) -> bool {
    let mut lhs = self.clone();
    let mut rhs = other.clone();

    lhs.set_offset(ZoneOffset::utc());
    rhs.set_offset(ZoneOffset::utc());

    lhs.raw == rhs.raw
  }
}

impl Eq for MinInstant {}

impl PartialOrd for MinInstant {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    let mut lhs = self.clone();
    let mut rhs = other.clone();

    lhs.set_offset(ZoneOffset::utc());
    rhs.set_offset(ZoneOffset::utc());

    Some(lhs.raw.cmp(&rhs.raw))
  }
}

impl Ord for MinInstant {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.partial_cmp(other).expect("PartialOrd for MinInstant is impl'd")
  }
}

/// minutes since start of the day. TODO.

/// An [inslusive, exclusive) time interval, with its `start` and `end` marked
/// by `MinInstant`. This interval must be non-negative.
pub struct MinInterval {
  start: MinInstant,
  end: MinInstant,
}

// TODO still contains magic number
// TODO improve human interaction
impl MinInstant {
  /// Constructs a MinInstant using current system time. Sets to UTC zone
  /// by default.
  pub fn now() -> Self {
    let t: i64 = Instant::now().seconds() / SEC_IN_MIN;

    if t > MINUTE_UPPERBOUND {
      panic!("datetime seconds overflowed")
    };
    if t < MINUTE_LOWERBOUND {
      panic!("datetime seconds negative")
    };

    Self {
      raw: t as u32,
      offset: ZoneOffset::utc(),
    }
  }

  /// Adjust by an input offset. This merely changes the timezone
  /// representation, and does not shift the represented time instance.
  ///
  /// Note that overflows are not possible due to how the types `MinInstant`
  /// and `ZoneOffset` are constructed.
  pub fn set_offset(&mut self, tgt_offset: ZoneOffset) {
    let diff = tgt_offset.raw() - self.offset.raw();

    // adjust timezone
    self.offset = tgt_offset;

    // adjust time in the same amount as timezone
    if diff >= 0 {
      self.raw += diff as u32;
    } else {
      self.raw -= (-diff) as u32;
    }
  }

  /// Returns the raw value of such a MinInstant, ie. a `u32` representing the
  /// number of minutes since Unix Epoch.
  pub fn raw(self) -> u32 {
    self.raw
  }

  /// Given a `Date`, converts it to corresponding `MinInstant` with UTC offset.
  /// Returns an error on u32 overflow.
  pub fn from_date(date: Date) -> year::Result<Self> {
    let yrs_min = date.get_yr().to_unix().num_min_since_epoch()?;
    let mons_min = date.get_mon().num_min_since_new_year(&date.get_yr() as &dyn Year);
    let days_min = (date.get_day() - 1) * MIN_IN_DAY;
    let hrs_min = (date.get_hr()) * MIN_IN_HR;
    let min_min = date.get_min();

    let arr_to_safely_sum = &[yrs_min, mons_min, days_min, hrs_min, min_min];
    let ret_opt = u32_safe_sum(arr_to_safely_sum);

    match ret_opt {
      Some(n) => Ok(MinInstant { raw: n, offset: ZoneOffset::utc() }),
      None => Err(YearError::DateToMinInstantOverFlow(
        date.get_yr().to_ce().raw(), 
        date.get_mon() as u32, 
        date.get_day(),
      ))
    }
  }
}

impl MinInterval {
  /// Constructs a `MinInterval` via a pair of `MinInstant`, which represents
  /// the start and end time. This constructor ensures non-negativity.
  pub fn new(start: MinInstant, end: MinInstant) -> MinInterval {
    MinInterval { start, end }
  }

  /// Constructs a `MinInterval` via a `MinInstant`, which represents its
  /// starting time, and some `u32`, which represents the duration in minutes
  /// of such interval.
  pub fn from_instance_and_minute_duration(
    mi: MinInstant,
    duration_minute: u32,
  ) -> MinInterval {
    let offset = mi.offset;
    MinInterval {
      start: mi,
      end: MinInstant {
        raw: mi.raw + duration_minute,
        offset,
      },
    }
  }
}

impl std::fmt::Display for MinInstant {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", Date::from_min_instant(*self).to_string())
  }
}

impl std::fmt::Display for MinInterval {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "({} - {})", self.start, self.end)
  }
}

#[derive(Debug)]
pub struct Date {
  yr: CeYear,
  mon: Month,
  day: u32,
  hr: u32,
  min: u32,
}

#[derive(Debug)]
pub enum DateError {}

// todo check overflow bounds
// todo fix timezone type defn
impl Date {
  /// Given a MinInstant, convert it to human-readable calendar form.
  ///
  /// Note that such a conversion takes into account the timezone offset of
  /// the provided MinInstant.
  pub fn from_min_instant(mi: MinInstant) -> Self {
    let (mut curr_year, mut curr_month) = (
      UnixYear::new(0).expect("Should be able to construct unix year 0"),
      Month::Jan,
    );
    let mut t = mi.raw();

    // strip year from t
    loop {
      let x = curr_year.num_min();
      if t >= x {
        (curr_year, t) = (
          curr_year
            .next()
            .expect("Year should not run out before MinInstant"),
          t - x,
        )
      } else {
        break;
      }
    }

    loop {
      // strip month from t
      let x = curr_month.num_min(&curr_year);
      if t >= x {
        (curr_month, t) = (
          curr_month.next().expect("Month Overflow"), 
          t - x
        )
      } else {
        break;
      }
    }

    Date {
      yr: curr_year.to_ce(),
      mon: Month::Jan,
      day: 1 + t / MIN_IN_DAY,
      hr: (t % MIN_IN_DAY) / MIN_IN_HR,
      min: t % MIN_IN_HR,
    }
  }

  pub fn get_yr(&self) -> CeYear { self.yr.clone() }
  pub fn get_mon(&self) -> Month { self.mon }
  
  /// Day in month, starts from 1.
  pub fn get_day(&self) -> u32 { self.day }
  pub fn get_hr(&self) -> u32 { self.hr }
  pub fn get_min(&self) -> u32 { self.min }
}

impl std::fmt::Display for Date {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}/{:?}/{} {}:{}",
      self.yr.raw(),
      self.mon,
      self.day,
      self.hr,
      self.min,
    )
  }
}

#[allow(unused_imports)]
mod test {
  use crate::time::timezone::ZoneOffset;

  use super::{Date, MinInstant};

  #[test]
  fn instant_to_date() {
    let mi = MinInstant {
      raw: 27905591,
      offset: ZoneOffset::utc(),
    };
    assert_eq!(
      "2023/Jan/21 21:11",
      format!("{}", Date::from_min_instant(mi))
    );
  }

  #[test]
  #[should_panic]
  fn mininstant_construction_contraint() {
    todo!() // shall be implemented when
            // MinInstant has a user-input constructor.
  }

  #[test]
  fn set_offset_overflow() {
    let mut mi = MinInstant {
      raw: 27905591,
      offset: ZoneOffset::utc(),
    };

    mi.set_offset(ZoneOffset::new(-300).unwrap());
    assert_eq!(mi.raw(), 27905591 - 300);
  }

  #[test]
  fn mininstant_date_conversions() {
    let mi = MinInstant {
      raw: 27905591,
      offset: ZoneOffset::utc(),
    };
    let mi2 = MinInstant::from_date(Date::from_min_instant(mi)).unwrap();
    println!("{} vs {}", mi.raw(), mi2.raw());
    assert_eq!(mi, mi2);
  }

  #[test]
  fn mininstant_order() {
    // Note that this test must occur at no earlier than 2023/Jan/21 21:11 
    // in order to produce intended result.
    let mi = MinInstant {
      raw: 27905591,
      offset: ZoneOffset::utc(),
    };

    let mi_now = MinInstant::now();

    assert!(mi < mi_now);
  }
}
