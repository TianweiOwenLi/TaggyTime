use core::panic;
use datetime::Instant;

mod year;
use year::{UnixYear, Year};

mod month;
use month::Month;

mod fact;

mod timezone;

use self::{fact::*, timezone::ZoneOffset, year::NextableYear};

const SEC_IN_MIN: i64 = 60;

// these bounds prevent overflow during timezone adjustments.
const MINUTE_UPPERBOUND: i64 = u32::MAX as i64 + timezone::UTC_UB as i64;
const MINUTE_LOWERBOUND: i64 = u32::MIN as i64 + timezone::UTC_LB as i64;

/// minutes since Unix Epoch. This can be casted to a different timezone
/// by incrementing both raw and offset at the same time, without changing
/// the actual time instant being represented.
#[derive(Debug, Clone, Copy)]
pub struct MinInstant {
  raw: u32,
  offset: ZoneOffset,
}

/// minutes since start of the day. TODO.

/// An [inslusive, exclusive) time interval, with its `start` and `end` marked
/// by `MinInstant`.
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
}

impl std::fmt::Display for MinInstant {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", Date::from_min_instant(*self).to_string())
  }
}

#[derive(Debug)]
pub struct Date {
  yr: u16,
  mon: Month,
  day: u32,
  hr: u32,
  min: u32,
}

// todo check overflow bounds
// todo fix timezone type defn
impl Date {
  /// Given a MinInstant, convert it to human-readable calendar form.
  ///
  /// Note that such a conversion takes into account the timezone offset of
  /// the provided MinInstant.
  pub fn from_min_instant(mi: MinInstant) -> Self {
    let (mut curr_year, mut curr_month) = (UnixYear::new(0), Month::Jan);
    let mut t = mi.raw();
    println!("{}", t);

    loop {
      // strip year from t
      let x = curr_year.num_min();
      if t >= x {
        (curr_year, t) = (curr_year.next_year(), t - x)
      } else {
        break;
      }
    }

    loop {
      // strip month from t
      let x = curr_month.num_min(&curr_year);
      if t >= x {
        (curr_month, t) = (curr_month.next_month(), t - x)
      } else {
        break;
      }
    }

    Date {
      yr: curr_year.to_ce().raw(),
      mon: Month::Jan,
      day: 1 + t / MIN_IN_DAY,
      hr: (t % MIN_IN_DAY) / MIN_IN_HR,
      min: t % MIN_IN_HR,
    }
  }
}

impl std::fmt::Display for Date {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}/{:?}/{} {}:{}",
      self.yr, self.mon, self.day, self.hr, self.min,
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
}
