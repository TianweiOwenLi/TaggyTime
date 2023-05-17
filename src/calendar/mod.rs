use std::collections::HashMap;

use crate::time::MinInterval;

use self::{
  cal_event::Event,
  task::{ExpirableImpact, Task},
};

use serde::{Deserialize, Serialize};

pub mod cal_event;
pub mod task;

#[derive(Debug)]
pub enum CalError {
  KeyNotFound(String),
  DoubleInsert(String),
  NewnameUnavailable(String),
}

/// A wrapper around `HashMap<String, _>`.
#[derive(Serialize, Deserialize)]
pub struct NameMap<T> {
  contents: HashMap<String, T>,
}

impl<T> NameMap<T> {
  /// Checks whether some item has already been loaded.
  pub fn contains(&self, key: &str) -> bool {
    self.contents.contains_key(key)
  }

  /// Attempts to insert some key-value pairs. Returns error on double-insert.
  pub fn unique_insert(&mut self, key: &str, val: T) -> Result<(), CalError> {
    if self.contains(key) {
      Err(CalError::DoubleInsert(key.to_string()))
    } else {
      self.contents.insert(key.to_string(), val);
      Ok(())
    }
  }

  /// Gets mutable ref.
  pub fn get_mut(&mut self, key: &str) -> Option<&mut T> {
    self.contents.get_mut(key)
  }

  /// Removes some item.
  pub fn remove(&mut self, key: &str) -> Option<T> {
    self.contents.remove(key)
  }

  /// Returns a reference iterator
  pub fn iter(&self) -> std::collections::hash_map::Iter<String, T> {
    self.contents.iter()
  }
}

impl NameMap<Vec<Event>> {
  /// Computes the number of minutes overlapped with some `MinInterval`.
  fn overlap_miv(&self, miv: MinInterval) -> u32 {
    let mut ret: u32 = 0;
    for event_vec in self.contents.values() {
      for event in event_vec {
        ret =
          ret.checked_add(event.1.clone().overlap(miv)).expect("Overflowed");
      }
    }
    ret
  }

  /// Givent the collection of events, compute the relative impact of a task.
  pub fn impact(&self, todo: &Task) -> ExpirableImpact {
    let miv = MinInterval::from_now_till(todo.due);
    let total_time = miv.num_min();
    let occupied_time = self.overlap_miv(miv);
    let available_time = total_time - occupied_time;
    let needed_time = todo.get_remaining_workload().num_min();

    ExpirableImpact::from((needed_time as f32) / (available_time as f32))
  }

  /// Performs filtration across events.
  pub fn filter_events<F: Fn(&Event) -> bool>(&mut self, f: F) {
    for (_, v) in &mut self.contents {
      v.retain(&f);
    }
  }
}
