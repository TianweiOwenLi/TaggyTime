use super::{*, Date, year::*};

use Weekday::*;
const WEEKDAY_LIST: [Weekday; 7] = [MO, TU, WE, TH, FR, SA, SU];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Weekday {
  MO,
  TU,
  WE,
  TH,
  FR,
  SA,
  SU,
}

impl Weekday {

  /// Next weekday. Wraps around.
  pub fn next_wrap(&self) -> Self {
    let idx = *self as usize + 1;
    WEEKDAY_LIST[idx % 7]
  }

  /// Computes the weekday corresponding to `n` days after Thursday (which is 
  /// the weekday for Unix Epoch, ie. 1970.1.1).
  fn thursday_plus(n: usize) -> Self {
    let days_after_monday = (TH as usize) + n;
    WEEKDAY_LIST[days_after_monday % 7]
  }
}

impl From<&Date> for Weekday {

  /// Returns the weekday of some `Date`.
  fn from(value: &Date) -> Self {
    let mut past_year_iter = CeYear::new(1970).unwrap();
    let mut days_in_past_years: u32 = 0;

    while past_year_iter != value.get_yr() {
      days_in_past_years += past_year_iter.days_in_year();
      past_year_iter = past_year_iter.next()
        .expect("CeYear can never overflow before match");
    }

    // -1 is to account for the fact that day_in_yr() starts from 1.
    let total_days = days_in_past_years + value.day_in_yr() - 1;

    Weekday::thursday_plus(total_days as usize)
  }
}

#[allow(dead_code, unused_imports)]
mod test {
  use super::*;
  
  #[test]
  fn weekday_arithmetic() {
    assert_eq!(MO, Weekday::thursday_plus(1005));
  }

  #[test]
  fn iterate() {
    assert_eq!(TU, SA.next_wrap().next_wrap().next_wrap())
  }
}