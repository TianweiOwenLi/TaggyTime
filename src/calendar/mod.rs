use std::collections::HashMap;

use self::cal_event::Event;

pub mod cal_event;
pub mod task;

#[derive(Debug)]
pub enum CalError<'a>{
  RenameNonexist(&'a str),
}

pub struct Calendars {
  contents: HashMap<String, Vec<Event>>
}

impl Calendars {
  pub fn mk_empty() -> Self {
    Calendars { contents: HashMap::<String, Vec<Event>>::new() }
  }

  /// Checks whether a particular `.ics` file has already been loaded. 
  pub fn contains(&self, cal_name: &str) -> bool {
    self.contents.contains_key(cal_name)
  }

  /// Inserts some `.ics` file WITHOUT checking pre-existence. 
  pub fn force_insert(&mut self, cal_name: &str, events: Vec<Event>) {
    self.contents.insert(cal_name.to_string(), events);
  }

  /// Renames some loaded `.ics` file.
  pub fn rename<'a>(&mut self, old_name: &'a str, new_name: &str) 
  -> Result<(), CalError<'a>> {
    match self.contents.remove(old_name) {
      Some(v) => {
        self.contents.insert(new_name.to_string(), v);
        Ok(())
      }
      None => Err(CalError::RenameNonexist(old_name))
    }
  }
}
