use std::collections::HashMap;

use crate::{time::MinInterval, util_typs::percent::Percent};

use self::{cal_event::Event, task::Task};

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
  pub fn mk_empty() -> Self {
    NameMap {
      contents: HashMap::<String, T>::new(),
    }
  }

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

  /// Inserts WITHOUT checking pre-existence.
  pub fn force_insert(&mut self, key: &str, val: T) {
    self.contents.insert(key.to_string(), val);
  }

  /// Renames some item.
  pub fn rename<'a>(
    &mut self,
    old_key: &'a str,
    new_key: &str,
  ) -> Result<(), CalError> {
    match self.contents.remove(old_key) {
      Some(v) => {
        if self.contains(new_key) {
          return Err(CalError::NewnameUnavailable(new_key.to_string()));
        }
        self.contents.insert(new_key.to_string(), v);
        Ok(())
      }
      None => Err(CalError::KeyNotFound(new_key.to_string())),
    }
  }

  /// Gets immutable ref.
  pub fn get_ref(&self, key: &str) -> Option<&T> {
    self.contents.get(key)
  }

  /// Gets mutable ref.
  pub fn get_mut(&mut self, key: &str) -> Option<&mut T> {
    self.contents.get_mut(key)
  }

  /// Removes some item.
  pub fn remove(&mut self, key: &str) -> Option<T> {
    self.contents.remove(key)
  }
}

impl NameMap<Vec<Event>> {
  /// Computes the number of minutes overlapped with some `MinInterval`.
  fn overlap_miv(&self, miv: MinInterval) -> u32 {
    let mut ret: u32 = 0;
    for event_vec in self.contents.values() {
      for event in event_vec {
        ret = ret
          .checked_add(event.1.clone().overlap(miv))
          .expect("Overflowed");
      }
    }
    ret
  }

  /// Givent the collection of events, compute the relative impact of a task.
  pub fn impact(&self, todo: &Task) -> Percent {
    let miv = MinInterval::from_now_till(todo.due);
    let total_time = miv.num_min();
    let occupied_time = self.overlap_miv(miv);
    let available_time = total_time - occupied_time;
    let needed_time = todo.get_remaining_workload().num_min();

    Percent::try_from((needed_time as f32) / (available_time as f32))
      .expect("impact overflowed")
  }

  /// Performs filtration across events. 
  pub fn filter_events<F: Fn(&Event) -> bool>(&mut self, f: F) {
    for (_, v) in &mut self.contents {
      v.retain(&f);
    }
  }
}
