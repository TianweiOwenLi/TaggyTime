//! Loads various types of files.

use std::path::Path;

use crate::{
  calendar::cal_event::Event,
  ics_parser::{lex_and_parse, ICSProcessError},
  time::timezone::ZoneOffset, util::path2string,
};

/// Loads the given `.ics` file according to the default timezone offset.
pub fn load_schedule_ics<P: AsRef<Path>>(
  path: P,
  default_tz: ZoneOffset,
) -> Result<Vec<Event>, ICSProcessError> {
  if ! path.as_ref().ends_with(".ics") {
    return Err(ICSProcessError::NotIcsFile(path2string(&path)));
  } 
  let parse_result = lex_and_parse(path, default_tz)?;
  parse_result
    .content
    .into_iter()
    .map(Event::try_from)
    .collect()
}
