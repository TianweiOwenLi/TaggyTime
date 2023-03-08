use super::fact::*;

const UNIX_YEAR_MIN: u16 = u16::MIN;
const UNIX_YEAR_MAX: u16 = u16::MAX - UNIX_EPOCH_YR_RAW;
const CE_YEAR_MIN: u16 = u16::MIN + UNIX_EPOCH_YR_RAW;
const CE_YEAR_MAX: u16 = u16::MAX;

#[derive(Debug)]
pub enum YearError {
  UnixYearConstructorOverflow(u16),
  CeYearConstructorUnderflow(u16),
}

type Result<T> = core::result::Result<T, YearError>;

pub enum YearLength {
  Leap,
  Common,
}

pub trait Year {
  /// converts a Year to a CeYear
  fn to_ce(&self) -> CeYear;

  /// converts a Year to a UnixYear
  fn to_unix(&self) -> UnixYear;

  /// get the length of year, either Leap or Common.
  fn get_year_length(&self) -> YearLength {
    let CeYear(n) = self.to_ce();
    if (n % 400 == 0) || (n % 4 == 0 && n % 100 != 0) {
      YearLength::Leap
    } else {
      YearLength::Common
    }
  }

  /// get the number of days in a year, either 366 for a Leap year or
  /// 365 for a Common year.
  fn days_in_year(&self) -> u32 {
    match self.get_year_length() {
      YearLength::Leap => 366,
      YearLength::Common => 365,
    }
  }

  /// the number of minutes in a year.
  #[inline(always)]
  fn num_min(&self) -> u32 {
    self.days_in_year() * MIN_IN_DAY
  }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnixYear(u16);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CeYear(u16);

impl UnixYear {
  pub fn new(n: u16) -> Result<Self> {
    if n > UNIX_YEAR_MAX {
      Err(YearError::UnixYearConstructorOverflow(n))
    } else {
      Ok(UnixYear(n))
    }
  }

  /// get the number of minutes from unix epoch to beginning of year.
  pub fn num_min_since_epoch(&self) -> u32 {
    match self.prev() {
      Some(prev_yr) => prev_yr.num_min_since_epoch() + prev_yr.num_min(),
      None => 0,
    }
  }

  pub fn next(&self) -> Option<Self> {
    let raw = self.0.checked_add(1)?;
    match UnixYear::new(raw) {
      Ok(uy) => Some(uy),
      _ => None,
    }
  }

  pub fn prev(&self) -> Option<Self> {
    let raw = self.0.checked_sub(1)?;
    Some(UnixYear::new(raw).expect("unixyear chk'd prev can never underflow"))
  }
}

impl CeYear {
  pub fn new(n: u16) -> Result<Self> {
    if n < CE_YEAR_MIN {
      Err(YearError::CeYearConstructorUnderflow(n))
    } else {
      Ok(CeYear(n))
    }
  }

  pub fn raw(&self) -> u16 {
    self.0
  }
}

impl Year for UnixYear {
  fn to_ce(&self) -> CeYear {
    CeYear::new(self.0 + UNIX_EPOCH_YR_RAW)
      .expect("Year module min/max has logical error")
  }

  fn to_unix(&self) -> UnixYear {
    UnixYear::new(self.0)
      .expect("Year module min/max has logical error")
  }
}

impl Year for CeYear {
  fn to_ce(&self) -> CeYear {
    CeYear::new(self.0)
      .expect("Year module min/max has logical error")
  }

  fn to_unix(&self) -> UnixYear {
    UnixYear::new(self.0 - UNIX_EPOCH_YR_RAW)
      .expect("Year module min/max has logical error")
  }
}


#[allow(dead_code, unused_imports)]
mod test {
  use super::*;

  #[test]
  fn ce_constructor_bound_chk() {
    assert!(CeYear::new(233).is_err());
    assert!(UnixYear::new(65500).is_err());
  }

  #[test]
  fn unix_prev_iterate() {
    let y_1973 = CeYear::new(1973).unwrap().to_unix();
    let y_1972 = y_1973.prev().unwrap();
    let y_1971 = y_1972.prev().unwrap();
    let y_1970 = y_1971.prev().unwrap();
    assert!(y_1970.prev().is_none());
  }

  #[test]
  fn convert_back_and_forth() {
    let origin_u = UnixYear::new(1999).unwrap();
    assert_eq!(
      origin_u, 
      origin_u.next().unwrap().to_ce().to_unix().prev().unwrap()
    )
  }

  #[test]
  fn comparsion() {
    assert!(UnixYear::new(2000).unwrap() < UnixYear::new(55555).unwrap());
    assert!(CeYear::new(6666).unwrap() > CeYear::new(2033).unwrap());
    assert_eq!(CeYear::new(1985).unwrap(), UnixYear::new(15).unwrap().to_ce());
  }
}
