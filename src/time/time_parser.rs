//! Parses strings to all kinds of times.

use super::fact::{HR_IN_DAY, MIN_IN_HR};
use super::{
  month::Month,
  year::{CeYear, Year},
};
use super::{timezone::ZoneOffset, MinInstant, TimeError};

// Attempts to parse some expression as u16.
fn parse_u16(expr: &str) -> Result<u16, TimeError> {
  match expr.parse() {
    Ok(n) => Ok(n),
    _ => Err(TimeError::NanErr(expr.to_string())),
  }
}

/// Attempts to parse some expression as u32.
pub fn parse_u32(expr: &str) -> Result<u32, TimeError> {
  match expr.parse() {
    Ok(n) => Ok(n),
    _ => Err(TimeError::NanErr(expr.to_string())),
  }
}

/// Attempts to parse some expression as f32.
pub fn parse_f32(expr: &str) -> Result<f32, TimeError> {
  match expr.parse() {
    Ok(n) => Ok(n),
    _ => Err(TimeError::NafErr(expr.to_string())),
  }
}

/// Parses some dynamically-ranged u32. Note that `lb` and `ub` are inclusive.
pub fn parse_u32_bound(expr: &str, lb: u32, ub: u32) -> Result<u32, TimeError> {
  let n = parse_u32(expr)?;
  if n >= lb && n <= ub {
    Ok(n)
  } else {
    Err(TimeError::NumOutOfBoundsErr(n))
  }
}

/// Parses some str as year, month, and day.
pub fn parse_ymd(
  expr: &str,
  tz: ZoneOffset,
) -> Result<(CeYear, Month, u32), TimeError> {
  let args: Vec<&str> = expr.split("/").map(|s| s.trim()).collect();
  match args[..] {
    [y, m, d] => {
      let y: CeYear = CeYear::new(parse_u16(y)?)?;
      let m: Month = m.parse()?;
      let d = parse_u32_bound(d, 1, m.num_days(&y))?;
      Ok((y, m, d))
    }
    [m, d] => {
      let y: CeYear = MinInstant::now(tz).decomp_yr_min().0.to_ce();
      let m: Month = m.parse()?;
      let d = parse_u32_bound(d, 1, m.num_days(&y))?;
      Ok((y, m, d))
    }
    _ => todo!(),
  }
}

/// Given some str, parses as a pair of hour and minute. If minute does not
/// exist, defaults to zero. If fails to parse or out-of-bound, returns error.
pub fn parse_hr_min(expr: &str) -> Result<(u32, u32), TimeError> {
  let (h, m) = match expr.split_once(':') {
    Some((hr_str, min_str)) => (parse_u32(hr_str)?, parse_u32(min_str)?),
    None => (parse_u32(expr)?, 0), // no min field, only hours
  };

  if h < HR_IN_DAY && m < MIN_IN_HR {
    Ok((h, m))
  } else {
    Err(TimeError::TimeParseErr(expr.to_string()))
  }
}
