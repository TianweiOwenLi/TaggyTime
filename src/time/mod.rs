
use datetime::Instant;
use core::panic;

mod year;
use year::{Year, UnixYear};

mod month;
use month::Month;

mod fact;

use self::{year::NextableYear, fact::*};

const SEC_IN_MIN: i64 = 60;
const MINUTE_UPPERBOUND: i64 = 0x7fffffff;
const MINUTE_LOWERBOUND: i64 = 0;

// minute since unix epoch
#[derive(Debug, Clone, Copy)]
pub struct MinInstant(u32);

// TODO still contains magic number
// TODO improve human interaction
impl MinInstant {
  pub fn now(offset_minute: i32) -> Self {
    let t: i64 = Instant::now().seconds() / SEC_IN_MIN;

    if t > MINUTE_UPPERBOUND {panic!("datetime seconds overflowed")};
    if t < MINUTE_LOWERBOUND {panic!("datetime seconds negative")};
    if offset_minute > 240 || offset_minute < -240 {
      panic!("offset too significant")
    }
    
    Self(t as u32)
  }

  pub fn raw(self) -> u32 {
    self.0
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

  /// given a MinInstant, convert it to human-readable calendar form.
  pub fn from_min_instant(mi: MinInstant) -> Self {
    let (mut curr_year, mut curr_month) 
      = (UnixYear::new(0), Month::Jan);
    let mut t = mi.raw();
    println!("{}", t);

    loop { // strip year from t
      let x = curr_year.num_min();
      if t >= x {
        (curr_year, t) = (curr_year.next_year(), t - x)
      } else {
        break
      }
    };

    loop { // strip month from t
      let x = curr_month.num_min(&curr_year);
      if t >= x {
        (curr_month, t) = (curr_month.next_month(), t - x)
      } else {
        break
      }
    };


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
      self.yr,
      self.mon,
      self.day,
      self.hr,
      self.min,
    )
  }
}

mod test {
  use super::{Date, MinInstant};

  #[test]
  fn test_instant_to_date() {
    assert_eq!(
      "2023/Jan/21 21:11", 
      format!("{}", Date::from_min_instant(MinInstant(27905591)))
    );
  }
}
