use std::collections::HashMap;

use crate::{time::MinInterval, util_typs::percent::Percent};

use self::{cal_event::Event, task::Task};

pub mod cal_event;
pub mod task;

#[derive(Debug)]
pub enum CalError<'a>{
  RenameNonexist(&'a str),
}

/// A wrapper around `HashMap<String, _>`.
pub struct NameMap<T> {
  contents: HashMap<String, T>
}

impl<T> NameMap<T> {
  pub fn mk_empty() -> Self {
    NameMap { contents: HashMap::<String, T>::new() }
  }

  /// Checks whether a particular `.ics` file has already been loaded. 
  pub fn contains(&self, key: &str) -> bool {
    self.contents.contains_key(key)
  }

  /// Inserts some `.ics` file WITHOUT checking pre-existence. 
  pub fn force_insert(&mut self, key: &str, events: T) {
    self.contents.insert(key.to_string(), events);
  }

  /// Renames some loaded `.ics` file.
  pub fn rename<'a>(&mut self, old_key: &'a str, new_key: &str) 
  -> Result<(), CalError<'a>> {
    match self.contents.remove(old_key) {
      Some(v) => {
        self.contents.insert(new_key.to_string(), v);
        Ok(())
      }
      None => Err(CalError::RenameNonexist(old_key))
    }
  }

  /// Removes some calendar.
  pub fn remove(&mut self, key: &str) {
    self.contents.remove(key);
  }
}

impl NameMap<Vec<Event>> {
  /// Computes the number of minutes overlapped with some `MinInterval`.
  fn overlap_miv(&self, miv: MinInterval) -> u32 {
    let mut ret: u32 = 0;
    for event_vec in self.contents.values() {
      for event in event_vec {
        ret = ret.checked_add(event.1.clone().overlap(miv)).expect("Overflowed");
      }
    }
    ret
  }

  pub fn impact(&self, todo: Task) -> Percent {
    let miv = MinInterval::from_now_till(todo.due);
    let total_time = miv.num_min();
    let occupied_time = self.overlap_miv(miv);
    let available_time = total_time - occupied_time;
    let needed_time = todo.get_remaining_workload().num_min();

    Percent::try_from((needed_time as f32) / (available_time as f32))
      .expect("impact overflowed")
  }
}
