//! Loads various types of files.

use crate::{
  calendar::cal_event::Event,
  ics_parser::{lex_and_parse, ICSProcessError},
  time::timezone::ZoneOffset,
};

/// Loads the given `.ics` file according to the default timezone offset.
pub fn load_schedule_ics(
  ics_filename: &str,
  default_tz: ZoneOffset,
) -> Result<Vec<Event>, ICSProcessError> {
  let parse_result = lex_and_parse(ics_filename, default_tz)?;
  parse_result
    .content
    .into_iter()
    .map(Event::try_from)
    .collect()
}
