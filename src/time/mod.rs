use core::panic;
use std::cmp::{min, max};
use datetime::Instant;

mod year;
use year::{UnixYear, Year, YearError};

mod month;

pub mod date;
use date::*;

pub mod week;

pub mod fact;

pub mod timezone;

use crate::{ics_parser::ICSProcessError, util_typs::RefinementError};

use self::{fact::*, timezone::ZoneOffset, year::CeYear};

// these bounds prevent overflow during timezone adjustments.
const MINUTE_UPPERBOUND: i64 = u32::MAX as i64 - timezone::UTC_UB as i64;
const MINUTE_LOWERBOUND: i64 = u32::MIN as i64 - timezone::UTC_LB as i64;

#[derive(Debug)]
pub enum TimeError {
  MinInstantAdvanceOverflow(u32, ZoneOffset, u32),
  MinInstantConstructionOverflow(u32),
  MinInstantConstructionUnderflow(u32),
  RefinementErr(RefinementError),
  TimeParseErr(String),
  TimeZoneParseErr(String),
}

impl From<RefinementError> for TimeError {
  fn from(value: RefinementError) -> Self {
    Self::RefinementErr(value)
  }
}

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
    self
      .partial_cmp(other)
      .expect("PartialOrd for MinInstant is impl'd")
  }
}

/// minutes since start of the day. TODO.

/// An [inslusive, exclusive) time interval, with its `start` and `end` marked
/// by `MinInstant`. This interval must be non-negative.
#[derive(Clone, Copy)]
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

  pub fn from_raw(raw: u32) -> Result<Self, TimeError> {
    if i64::from(raw) > MINUTE_UPPERBOUND {
      return Err(TimeError::MinInstantConstructionOverflow(raw));
    };
    if i64::from(raw) < MINUTE_LOWERBOUND {
      return Err(TimeError::MinInstantConstructionUnderflow(raw));
    };

    Ok(Self { raw, offset: ZoneOffset::utc()})
  }

  /// Returns the current offset.
  pub fn get_offset(&self) -> ZoneOffset {
    self.offset
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
  pub fn from_date(date: &Date) -> year::Result<Self> {
    let yrs_min = date.get_yr().to_unix().num_min_since_epoch()?;
    let mons_min = date
      .get_mon()
      .num_min_since_new_year(&date.get_yr() as &dyn Year);
    let days_min = (date.day_in_mon() - 1) * MIN_IN_DAY; // 1-index to 0-index
    let hrs_min = (date.get_hr()) * MIN_IN_HR;
    let min_min = date.get_min();

    let arr_to_safely_sum = &[yrs_min, mons_min, days_min, hrs_min, min_min];
    let ret_opt = u32_safe_sum(arr_to_safely_sum);

    match ret_opt {
      Some(n) => Ok(MinInstant {
        raw: n,
        offset: ZoneOffset::utc(),
      }),
      None => Err(YearError::DateToMinInstantOverFlow(
        date.get_yr().to_ce().raw(),
        date.get_mon() as u32,
        date.day_in_mon(),
      )),
    }
  }

  /// Advances the `MinInstant` by given number of minutes. Checks bounds while
  /// advancing, and returns an error if overflows.
  pub fn advance(&self, num_min: u32) -> Result<MinInstant, TimeError> {
    let added_raw = self.raw.checked_add(num_min);
    if let Some(added_safe_raw) = added_raw {
      let zoneoffset_redundancy = MINUTE_UPPERBOUND
        .checked_add(self.offset.raw())
        .expect("MI upperbound shall never overflow when added by zone offset");
      if i64::from(added_safe_raw) <= zoneoffset_redundancy {
        return Ok(MinInstant {
          raw: added_safe_raw,
          offset: self.offset,
        });
      }
    }
    Err(TimeError::MinInstantAdvanceOverflow(
      self.raw,
      self.offset,
      num_min,
    ))
  }
}

impl MinInterval {
  /// Constructs a `MinInterval` via a pair of `MinInstant`, which represents
  /// the start and end time. This constructor ensures non-negativity.
  pub fn new(start: MinInstant, end: MinInstant) -> MinInterval {
    MinInterval { start, end }
  }

  /// Creates a `MinInterval` from now till the given `MinInstant`.
  pub fn from_now_till(end: MinInstant) -> MinInterval {
    MinInterval { start: MinInstant::now(), end }
  }

  /// Computes the duration of overlap of two `MinInterval` in minutes.
  pub fn overlap_duration(&self, rhs: MinInterval) -> u32 {
    let (lb, ub) = (max(self.start, rhs.start), min(self.end, rhs.end));
    if lb >= ub { 0 } else { ub.raw - lb.raw }
  }

  /// Converts start and end to `Date` and prints accordingly
  pub fn as_date_string(&self) -> String {
    let start_str = Date::from_min_instant(self.start);
    let end_str = Date::from_min_instant(self.end);
    format!("{} - {}", start_str, end_str)
  }

  /// Advances the `MinInterval` by given number of minutes. Checks bounds while
  /// advancing, and returns an error if overflows.
  pub fn advance(&self, num_min: u32) -> Result<MinInterval, TimeError> {
    Ok(MinInterval {
      start: self.start.advance(num_min)?,
      end: self.end.advance(num_min)?,
    })
  }

  /// Advances the `MinInterval` by given number of minutes. Checks bounds while
  /// advancing. Panics if encounters overflow.
  pub fn advance_unwrap(&self, num_min: u32) -> MinInterval {
    self.advance(num_min).unwrap()
  }

  /// Advances the `MinInterval` until its starting time matches the
  /// provided `DateProperty`, or if `start` exceeds the `until` mininstant.
  pub fn advance_until(
    &self,
    dp: &DateProperty,
    until_opt: Option<MinInstant>,
  ) -> Result<Option<MinInterval>, TimeError> {
    let mut new_miv = self.clone();
    match until_opt {
      Some(until) => {
        while !dp.check(Date::from_min_instant(new_miv.get_start())) {
          new_miv = new_miv.advance(MIN_IN_DAY)?;
          if new_miv.start > until { return Ok(None); }
        }
      }
      None => {
        while !dp.check(Date::from_min_instant(new_miv.get_start())) {
          new_miv = new_miv.advance(MIN_IN_DAY)?;
        }
      }
    }
    Ok(Some(new_miv))
  }

  /// Advances the `MinInterval` until its starting time matches the
  /// provided `DateProperty`, or if `start` exceeds the `until` mininstant. 
  /// 
  /// [note] This function panics on overflow.
  pub fn advance_until_unwrap(
    &self,
    dp: &DateProperty,
    until_opt: Option<MinInstant>,
  ) -> Option<MinInterval> {
    self.advance_until(dp, until_opt).unwrap()
  }

  pub fn get_start(&self) -> MinInstant {
    self.start
  }

  pub fn get_end(&self) -> MinInstant {
    self.end
  }

  pub fn num_min(&self) -> u32 {
    self.end.raw - self.start.raw
  }

}

impl std::fmt::Display for MinInstant {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "mi({})", self.raw)
  }
}

impl std::fmt::Display for MinInterval {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "({} - {})", self.start, self.end)
  }
}

// -------------------------------- Utilities --------------------------------

/// Attempts to parse some expression as u32.
fn parse_u32(expr: &str) -> Result<u32, TimeError> {
  match expr.parse() {
    Ok(n) => Ok(n),
    _ => Err(TimeError::TimeZoneParseErr(expr.to_string()))
  }
}

/// Given some str, parses as a pair of hour and minute. If minute does not 
/// exist, defaults to zero.
fn parse_hr_min(expr: &str) -> Result<(u32, u32), TimeError> {
  let (h, m) = match expr.split_once(':') {
    Some((hr_str, min_str)) => (parse_u32(hr_str)?, parse_u32(min_str)?),
    None => (parse_u32(expr)?, 0) // no min field, only hours
  };
  
  if h < 24 && m < 60 { 
    Ok((h, m)) 
  } else { 
    Err(TimeError::TimeZoneParseErr(expr.to_string())) 
  }
}

#[allow(unused_imports)]
mod test {
  use crate::time::{month::Month, timezone::ZoneOffset, year::CeYear};

  use super::{Date, MinInstant, MinInterval};

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
    let mi2 = MinInstant::from_date(&Date::from_min_instant(mi)).unwrap();
    assert_eq!(mi, mi2);
  }

  #[test]
  // Note that this test must occur at no earlier than 2023/Jan/21 21:11
  // in order to produce intended result.
  fn mininstant_order() {
    let mi = MinInstant {
      raw: 27905591,
      offset: ZoneOffset::utc(),
    };

    let mi_now = MinInstant::now();

    assert!(mi < mi_now);
  }

  #[test]
  fn mi_eq() {
    let m1 = MinInstant { raw: 5000, offset: ZoneOffset::utc() };
    let m2 = MinInstant { raw: 5060, offset: ZoneOffset::new(60).unwrap() };
    assert_eq!(m1, m2);
  }

  #[test]
  /// This test guarantees that u32 parses work as intended even with
  /// leading zeroes.
  fn parse_u32_behavior() {
    let parsed: u32 = "0002333".parse().unwrap();
    assert_eq!(2333, parsed);
  }

  #[test]
  fn miv_overlap() {
    let offset = ZoneOffset::utc();
    let t1 = MinInstant { raw: 23333, offset };
    let t2 = MinInstant { raw: 23300, offset };
    let t3 = MinInstant { raw: 5000, offset };
    let t4 = MinInstant { raw: 40000, offset };

    let miv_1 = MinInterval::new(t1, t2);
    assert_eq!(0, miv_1.overlap_duration(miv_1));

    let miv_2 = MinInterval::new(t3, t4);
    let miv_3 = MinInterval::new(t2, t1);
    assert_eq!(23333 - 23300, miv_2.overlap_duration(miv_3));
    assert_eq!(23333 - 23300, miv_3.overlap_duration(miv_2));

    let miv_4 = MinInterval::new(t3, t1);
    let miv_5 = MinInterval::new(t2, t4);
    assert_eq!(23333 - 23300, miv_5.overlap_duration(miv_4));
  }
}
