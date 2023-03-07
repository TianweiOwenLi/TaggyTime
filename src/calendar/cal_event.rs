use crate::percent::Percent;
use crate::time::{MinInstant, MinInterval};

/// Indicators representing subsets of days in a week.
pub struct WeekdayBitset {
  mon: bool,
  tue: bool,
  wed: bool,
  thu: bool,
  fri: bool,
  sat: bool,
  sun: bool,
}

impl WeekdayBitset {
  /// Unset all days of the week.
  pub fn clear(&mut self) {
    self.mon = false;
    self.tue = false;
    self.wed = false;
    self.thu = false;
    self.fri = false;
    self.sat = false;
    self.sun = false;
  }

  /// Checks whether none of the says in a week is selected.
  pub fn is_empty(&self) -> bool {
    self.mon && self.tue && self.wed && self.thu && self.fri && self.sat && self.sun
  }
}

pub struct WeekdayRecurrence {
  start: MinInstant,
  end: MinInstant,
  repeat: WeekdayBitset,
  interval: MinInterval,
}

/// Describes when shall some recurring events happen.
///
/// [todo] Implement custom executable functions that describes a recurrence.
pub enum Recurrence {
  Once(MinInterval),
  Weekly(MinInstant, WeekdayBitset, MinInterval),
}

/// A wrapper around u32, which represents the number of minutes needed to
/// complete some task. Such a u32 can only be from 0 to 60,000 (inclusive)
/// to prevent u32 multiplication overflow.
///
/// # Examples
/// ```
/// let w1: Workload = Workload::from_num_min(16);
/// let w1: Workload = Workload::from_num_min(15);
///
/// let p = Percent::from_u8(63);
///
/// let d1 = w1.get_duration(); // = 16
/// let d2 = w2
///   .multiply_percent(p)
///   .get_duration(); // 25 * (63%) = 15.75, which rounds to 16.
///
/// assert_eq!(d1, d2);
/// ```
pub struct Workload(u32);

impl Workload {
  /// Construct a `Workload` instance from some `u32`, which represents the
  /// number of minutes of such a workload. Only values from 0 to 60,000
  /// (inclusive) are allowed, in order to prevent u32 multiplication overflow.
  pub fn from_num_min(num_min: u32) -> Result<Self, String> {
    if num_min <= 60_000 {
      Ok(Workload(num_min))
    } else {
      Err("Workload is too high: cannot exceed 60,000 minutes".to_string())
    }
  }

  /// Multiply a Workload instance by some percentage. Rounded to the nearest
  /// integer minute.
  ///
  /// # Example
  /// ```
  /// assert_eq!(
  ///   31,
  ///   Workload(60).multiply_percent(Percent::from_u8(51))
  /// );
  /// ```
  pub fn multiply_percent(&self, p: Percent) -> Self {
    // will not overflow since such produce never exceeds 100 * 60_000.
    let workload_times_numerator = self.0 * (p.raw() as u32);

    let mut divided_by_denominator = workload_times_numerator / 100;

    // rounding up
    if workload_times_numerator % 100 >= 50 {
      divided_by_denominator += 1;
    }

    Workload(divided_by_denominator)
  }

  /// Returns the duration, in number of minutes, of such a workload.
  pub fn get_duration(&self) -> u32 {
    self.0
  }
}

impl std::fmt::Display for Recurrence {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Recurrence::Once(miv) => {
        write!(f, "Once {}", miv)
      }
      Recurrence::Weekly(start, repeat, miv) => {
        write!(f, "Repeat: ")
      }
    }
  }
}
