//! Structure that represents calendar days.

use crate::const_params::HANDLE_WKST;
use crate::ics_parser::ics_syntax::{Freq, FreqAndRRules, RRuleToks};
use crate::ics_parser::lexer::Token;
use crate::time::{month::Month, week::Weekday};

use crate::time::*;

use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

/// A struct that represents some time instance in human-readable form. Namely,
/// it has fields like year, month, day, hour, and minute.
///
/// Note that `Date` does not record any information about timezone.
#[derive(Debug, Clone, Copy)]
pub struct Date {
  pub yr: CeYear,
  pub mon: Month,
  pub day: u32,
  pub hr: u32,
  pub min: u32,
  pub tz: ZoneOffset,
}

// todo check overflow bounds
// todo fix timezone type defn
impl Date {
  /// Given a MinInstant, convert it to human-readable calendar form.
  ///
  /// Note that such a conversion takes into account the timezone offset of
  /// the provided MinInstant.
  pub fn from_min_instant(mi: MinInstant) -> Self {
  
    let (curr_year, mut t) = mi.decomp_yr_min();
    
    let mut curr_month = Month::Jan;
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
      tz: mi.offset,
    }
  }

  /// Constructs an instance of `Self` from two strings, one is of form
  /// `yyyymmdd`, and the other is of form `hhmmss`.
  pub fn from_ics_time_string(
    ymd: &str,
    hms: &str,
    tz: ZoneOffset,
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
    let mon_res: Result<u32, _> = mon_str.parse();
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

        Ok(Date {yr, mon, day, hr, min, tz})
      }
      _ => Err(ICSProcessError::ICSTimeMalformatted(
        ymd.to_string(),
        hms.to_string(),
      )),
    }
  }

  // ------------- The followings are all attribute functions.  -------------

  /// Day in year, starts from 1.
  pub fn day_in_yr(&self) -> u32 {
    let mut ret: u32 = self.day;
    let mut var_month = Month::Jan;
    while var_month != self.mon {
      ret += var_month.num_days(&self.yr);
      var_month = var_month
        .next()
        .expect("Month iterator can never run out before match");
    }
    ret
  }

  /// Given a default timezone, parses a string as a date.
  pub fn parse_from_str(s: &str, default_tz: ZoneOffset) -> Result<Self, TimeError> {
    let bad = Err(TimeError::DateParsingErr(s.to_string()));
    let args: Vec<&str> = s.split(' ').map(|s| s.trim()).collect();

    if args.len() >= 3 { return bad; } // too many items

    let tz = match args.get(2) {
      Some(s) => s.parse()?,
      None => default_tz,
    };

    match args[..2] {
      [ymd_str, time] => {
        let (yr, mon, day) = parse_ymd(ymd_str)?;
        let (hr, min) = parse_hr_min(time)?;
        Ok(Date { yr, mon, day, hr, min, tz })
      }
      _ => bad
    }
  }
}


impl std::fmt::Display for Date {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}/{:?}/{} {:02}:{:02}, tz={}",
      self.yr.raw(),
      self.mon,
      self.day,
      self.hr,
      self.min,
      self.tz,
    )
  }
}

use std::rc::Rc;
pub trait DatePropertyElt: From<Date> + Eq + Hash + std::fmt::Debug {}

pub struct DateProperty {
  filter_fn: Rc<dyn Fn(Date) -> bool>,
  dbg_info: String,
}

impl Clone for DateProperty {

  /// Makes a clone while shallow-copying the filter function.
  fn clone(&self) -> Self {
    DateProperty { 
      filter_fn: Rc::clone(&self.filter_fn),
      dbg_info: self.dbg_info.clone() 
    }
  }
}

impl DateProperty {
  pub fn check(&self, d: Date) -> bool {
    (self.filter_fn)(d)
  }

  pub fn always() -> Self {
    DateProperty { 
      filter_fn: Rc::new(|_| true), 
      dbg_info: String::from("[true]") 
    }
  }
}

impl<T: DatePropertyElt + 'static> From<Vec<T>> for DateProperty {
  fn from(value: Vec<T>) -> Self {
    let dbg_info = format!("{:?}", &value);
    let property_set = HashSet::<T>::from_iter(value.into_iter());
    DateProperty {
      filter_fn: Rc::new(move |d: Date| property_set.contains(&T::from(d))),
      dbg_info,
    }
  }
}

impl From<Vec<RRuleToks>> for DateProperty {
  
  /// [todo] consider restriction constraints as per RFC 5545.
  fn from(value: Vec<RRuleToks>) -> Self {
    let mut dp = DateProperty::always();
    let mut dp_is_always = true;

    for rrt in value {
      match rrt.tag {
        Token::BYDAY => {
          let v: Vec<Weekday> = rrt
            .content
            .iter()
            .map(|s| Weekday::from(s.as_str()))
            .collect();
          dp = if dp_is_always {
            dp_is_always = false;
            DateProperty::from(v)
          } else {
            dp * DateProperty::from(v)
          };
        }
        Token::BYHOUR | Token::BYMIN | Token::BYMONTH | Token::BYMONTHDAY 
        | Token::BYSETPOS | Token::BYWEEKNO | Token::BYYEARDAY => {
          unimplemented!()
        }
        Token::WKST => {
          if HANDLE_WKST {
            todo!("Needs to handle WKST tag")
          }
        }
        t => {
          unreachable!("Encountered unexpected rrule tag: {}", t)
        }
      }
    }
    dp
  }
}

impl std::fmt::Debug for DateProperty {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.dbg_info)
  }
}

impl std::ops::Mul for DateProperty {
  type Output = Self;
  fn mul(self, rhs: Self) -> Self::Output {
    DateProperty {
      filter_fn: Rc::new(move |x: Date| {
        (self.filter_fn)(x) && (rhs.filter_fn)(x)
      }),
      dbg_info: format!("{} and {}", self.dbg_info, rhs.dbg_info),
    }
  }
}

/// Strips the `BYDAY` property, ie. which days of a week, from some
/// `FreqAndRules` that is of variant `Freq::Weekly`.
///
/// [todo] Needs to be reimplemented sometime.
///
/// [todo] Does not yet faithfully show the rrule of weekly-no-pattern event.
pub fn parse_dateproperty_week(fr: &FreqAndRRules) -> DateProperty {
  let mut weekday_vec = Vec::<Weekday>::new();

  match fr.freq {
    Freq::Weekly => {
      'iter_recur_rules: for item in &fr.content {
        if let Token::BYDAY = &item.tag {
          for s in &item.content {
            weekday_vec.push(Weekday::from(s.as_str()));
          }
          break 'iter_recur_rules;
        }
      }
      return DateProperty::from(weekday_vec);
    }
    _ => unimplemented!(),
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
      tz: ZoneOffset::utc(),
    };
    assert_eq!(TU, Weekday::from(treeday));
  }

  #[test]
  fn ics_string_to_date() {
    let (ymd, hms) = ("20220314", "211123");
    let date1 = Date::from_ics_time_string(ymd, hms, ZoneOffset::utc()).unwrap();

    let date2 = Date {
      yr: CeYear::new(2022).unwrap(),
      mon: Month::Mar,
      day: 14,
      hr: 21,
      min: 11,
      tz: ZoneOffset::utc(),
    };

    let mi1 = MinInstant::from_date(&date1).unwrap();
    let mi2 = MinInstant::from_date(&date2).unwrap();

    assert_eq!(mi1.raw, mi2.raw);
  }

  #[test]
  fn yearday() {
    let treeday = Date {
      yr: CeYear::new(2100).unwrap(),
      mon: Month::Mar,
      day: 12,
      hr: 10,
      min: 05,
      tz: ZoneOffset::utc(),
    };

    assert_eq!(treeday.day_in_yr(), 31 + 28 + 12);
  }
}
