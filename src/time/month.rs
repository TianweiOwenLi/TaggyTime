use super::{fact::*, TimeError};
use super::year::{Year, YearLength};

use Month::*;
const MONTH_LIST: [Month; 12] =
  [Jan, Feb, Mar, Apr, May, Jun, Jul, Aug, Sep, Oct, Nov, Dec];

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Month {
  Jan = 0,
  Feb,
  Mar,
  Apr,
  May,
  Jun,
  Jul,
  Aug,
  Sep,
  Oct,
  Nov,
  Dec,
}

impl Month {
  pub fn next(&self) -> Option<Self> {
    MONTH_LIST.get(*self as usize + 1).copied()
  }

  pub fn prev(&self) -> Option<Self> {
    let safe_sub = (*self as usize).checked_sub(1)?;
    MONTH_LIST.get(safe_sub).copied()
  }

  pub fn num_days(&self, y: &dyn Year) -> u32 {
    use Month::*;
    if *self == Jan {
      // feb
      match y.get_year_length() {
        YearLength::Leap => 29,
        _ => 28,
      }
    } else if [Apr, Jun, Sep, Nov].contains(&self) {
      30
    } else {
      31
    }
  }

  pub fn num_min(&self, y: &dyn Year) -> u32 {
    self.num_days(y) * MIN_IN_DAY
  }

  /// Number of minutes from beginning of the given year to the
  /// beginning of the month.
  pub fn num_min_since_new_year(&self, y: &dyn Year) -> u32 {
    match self.prev() {
      Some(prev_mon) => prev_mon
        .num_min_since_new_year(y)
        .checked_add(prev_mon.num_min(y))
        .expect("Month is never large enough to let u32 overflow"),
      None => 0,
    }
  }
}

impl TryFrom<u32> for Month {
  type Error = TimeError;

  /// Tries to convert a usize to corresponding month, starting with zero.
  fn try_from(value: u32) -> Result<Self, Self::Error> {
    match value {
      0 => Ok(Jan), 
      1 => Ok(Feb), 
      2 => Ok(Mar), 
      3 => Ok(Apr), 
      4 => Ok(May), 
      5 => Ok(Jun), 
      6 => Ok(Jul), 
      7 => Ok(Aug), 
      8 => Ok(Sep), 
      9 => Ok(Oct), 
      10 => Ok(Nov), 
      11 => Ok(Dec), 
      n => Err(TimeError::MonthParseErr(value))
    }
  }
}

#[allow(dead_code, unused_imports)]
mod test {
  use super::*;

  #[test]
  fn prev_next_iterate() {
    let may = Month::May;

    let jun = may.next().unwrap();
    let jul = jun.next().unwrap();
    let aug = jul.next().unwrap();
    let sep = aug.next().unwrap();
    let oct = sep.next().unwrap();
    let nov = oct.next().unwrap();
    let dec = nov.next().unwrap();
    assert!(dec.next().is_none());

    let apr = may.prev().unwrap();
    let mar = apr.prev().unwrap();
    let feb = mar.prev().unwrap();
    let jan = feb.prev().unwrap();
    assert!(jan.prev().is_none());
  }
}
