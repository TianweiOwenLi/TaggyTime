
use super::year::{Year, YearLength};
use super::fact::*;

#[derive(PartialEq, Debug)]
pub enum Month {
  Jan, Feb, Mar, Apr, May, Jun,
  Jul, Aug, Sep, Oct, Nov, Dec,
}

impl Month {

  pub fn next_month(&self) -> Self {
    use Month::*;
    match self {
      Jan => Feb, Feb => Mar, Mar => Apr, Apr => May, 
      May => Jun, Jun => Jul, Jul => Aug, Aug => Sep,
      Sep => Oct, Oct => Nov, Nov => Dec, 
      _ => panic!("no month after december")
    }
  }

  fn num_days(&self, y: &dyn Year) -> u32 {
    use Month::*;
    if *self == Jan { // feb
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

}
