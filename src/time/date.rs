//! Structure that represents calendar days.

use crate::time::{month::Month, week::Weekday};

use crate::time::*;

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
        (curr_month, t) = (curr_month.next().expect("Month Overflow"), t - x)
      } else {
        break;
      }
    }

    Date {
      yr: curr_year.to_ce(),
      mon: curr_month,
      day: 1 + t / MIN_IN_DAY,
      hr: (t % MIN_IN_DAY) / MIN_IN_HR,
      min: t % MIN_IN_HR,
    }
  }

  /// Constructs an instance of `Self` from two strings, one is of form
  /// `yyyymmdd`, and the other is of form `hhmmss`.
  pub fn from_ics_time_string(
    ymd: &str,
    hms: &str,
  ) -> Result<Self, ICSProcessError> {
    // Return error message
    let bad = Err(ICSProcessError::ICSTimeMalformatted(
      ymd.to_string(),
      hms.to_string(),
    ));

    if ymd.len() < 8 || hms.len() < 6 {
      return bad;
    }

    let yr_str = &ymd[0..4];
    let mon_str = &ymd[4..6];
    let day_str = &ymd[6..8];
    let hr_str = &hms[0..2];
    let min_str = &hms[2..4];

    let yr_res: Result<u16, _> = yr_str.parse();
    let mon_res: Result<usize, _> = mon_str.parse();
    let day_res: Result<u32, _> = day_str.parse();
    let hr_res: Result<u32, _> = hr_str.parse();
    let min_res: Result<u32, _> = min_str.parse();

    match (yr_res, mon_res, day_res, hr_res, min_res) {
      (Ok(y), Ok(m), Ok(d), Ok(h), Ok(mi)) => {
        let yr = match CeYear::new(y) {
          Ok(y) => y,
          _ => return bad,
        };

        // since Month::try_from() is 0-indexed
        let mon = if let Some(m0) = m.checked_sub(1) {
          match Month::try_from(m0) {
            Ok(m) => m,
            _ => return bad,
          }
        } else {
          return bad;
        };

        let day = if d <= mon.num_days(&yr) && d > 0 {
          d
        } else {
          return bad;
        };

        let hr = if h <= 23 {
          h
        } else {
          return bad;
        };
        let min = if mi <= 59 {
          mi
        } else {
          return bad;
        };

        let ret_date = Date {yr, mon, day, hr, min};
        Ok(ret_date)
      }
      _ => Err(ICSProcessError::ICSTimeMalformatted(
        ymd.to_string(),
        hms.to_string(),
      )),
    }
  }


  // ------------- The followings are all attribute functions.  -------------

  pub fn get_yr(&self) -> CeYear {
    self.yr.clone()
  }
  pub fn get_mon(&self) -> Month {
    self.mon
  }

  /// Day in year, starts from 1.
  pub fn day_in_yr(&self) -> u32 {
    let mut ret: u32 = self.day_in_mon();
    let mut var_month = Month::Jan;
    while var_month != self.mon {
      ret += var_month.num_days(&self.yr);
      var_month = var_month.next()
        .expect("Month iterator can never run out before match");
    }
    ret
  }

  /// Day in month, starts from 1.
  pub fn day_in_mon(&self) -> u32 {
    self.day
  }

  pub fn day_in_week(&self) -> Weekday {
    Weekday::from(self)
  }

  pub fn get_hr(&self) -> u32 {
    self.hr
  }
  pub fn get_min(&self) -> u32 {
    self.min
  }
}

impl std::fmt::Display for Date {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}/{:?}/{} {:02}:{:02}",
      self.yr.raw(),
      self.mon,
      self.day,
      self.hr,
      self.min,
    )
  }
}

#[allow(dead_code, unused_imports)]
mod test {
  use super::*;

  #[test]
  fn calc_weekday() {
    use crate::time::month::Month;
    use Weekday::*;

    let treeday = Date {
      yr: CeYear::new(2023).unwrap(), 
      mon: Month::Mar, 
      day: 14, 
      hr: 21, 
      min: 11, 
    };
    assert_eq!(TU, Weekday::from(&treeday));
  }


  #[test]
  fn ics_string_to_date() {
    let (ymd, hms) = ("20220314", "211123");
    let date1 = Date::from_ics_time_string(ymd, hms).unwrap();

    let date2 = Date {
      yr: CeYear::new(2022).unwrap(), 
      mon: Month::Mar, 
      day: 14, 
      hr: 21, 
      min: 11, 
    };

    let mi1 = MinInstant::from_date(&date1).unwrap();
    let mi2 = MinInstant::from_date(&date2).unwrap();

    assert_eq!(mi1.raw(), mi2.raw());
  }

  #[test]
  fn yearday() {
    let treeday = Date {
      yr: CeYear::new(2100).unwrap(), 
      mon: Month::Mar, 
      day: 12, 
      hr: 10, 
      min: 05, 
    };

    assert_eq!(treeday.day_in_yr(), 31 + 28 + 12);
  }
}