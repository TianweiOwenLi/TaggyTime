
use crate::time::*;
use crate::percent::Percent;

/// A wrapper around u32, which represents the number of minutes needed to 
/// complete some task. Such a u32 can only be from 0 to 60,000 (inclusive) 
/// to prevent u32 multiplication overflow. 
/// 
/// # Examples
/// ```
/// let w1: Workload = Workload::from_num_min(16);
/// let w2: Workload = Workload::from_str("25");
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

/// A struct that represents some task to be done. 
/// 
/// This struct contains the following fields: 
/// 
/// `name`: a `String` representing the name of the task.
/// 
/// `due`: the due date of such a task, represented as a `MinInstant`.
/// 
/// `length`: number of minutes needed to complete such a task from scratch.
/// 
/// `completion`: the progress of such a task, in percentage.
/// 
/// `impact`: the ratio of completion time, relative to available time before 
/// deadline. Can only be updated with an external schedule.
/// 
/// [todo] Finish example code
/// 
/// # Examples
/// ```
/// 
/// ```
pub struct Todo {
  name: String,
  due: MinInstant,
  length: Workload,
  completion: Percent,
  impact: Percent,
}


pub struct TodoList {
  content: Vec<Todo>, 
}


impl Workload {

  /// Construct a `Workload` instance via attempting to trim and parse `&str` 
  /// into an `u32`, before calling `from_num_min()`. Only values from 0 
  /// to 60,000 (inclusive) are allowed, in order to prevent u32 multiplication 
  /// overflow.
  pub fn from_str(s: &str) -> Result<Self, String> {
    let ps: Result<u32, _>  = s.trim().parse::<u32>();
    if let Ok(n) = ps {
      Self::from_num_min(n)
    } else {
      Err(format!("Cannot parse Workload from &str: ''{}''", s))
    }
  }


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
    if workload_times_numerator % 100 >= 50 {divided_by_denominator += 1;}

    Workload(divided_by_denominator)
  }


  /// Returns the duration, in number of minutes, of such a workload.
  pub fn get_duration(&self) -> u32 { self.0 }

}


impl Todo {

  /// Computes the remaining workload of this `Todo` item, considering its 
  /// `length` and `completion` fields.
  pub fn get_remaining_workload(&self) -> Workload {
    self.length.multiply_percent(self.completion.one_minus())
  }
}


impl TodoList {

  

}


#[allow(unused_imports)]
mod test {
  use super::*;

  fn nada() {
    let td = Todo {
      name: "Name".to_string(),
      due: MinInstant::now(0),
      length: Workload::from_num_min(60).unwrap(),
      completion: Percent::from_u8(0).unwrap(),
      impact: Percent::from_u8(2).unwrap(),
    };

  }
}
