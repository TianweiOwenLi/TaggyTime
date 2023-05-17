//! Loads various types of files.

use std::path::Path;

use crate::{
  calendar::cal_event::Event,
  ics_parser::{lex_and_parse, ICSProcessError},
  time::timezone::ZoneOffset,
  util::path2string,
};

/// Loads the given `.ics` file according to the default timezone offset.
pub fn load_schedule_ics<P: AsRef<Path>>(
  path: P,
  default_tz: ZoneOffset,
) -> Result<Vec<Event>, ICSProcessError> {
  let bad_extension = Err(ICSProcessError::NotIcsFile(path2string(&path)));
  match path.as_ref().extension() {
    Some(ext) => {
      if ext != "ics" {
        return bad_extension;
      }

      let parse_result = lex_and_parse(path, default_tz)?;
      parse_result.content.into_iter().map(Event::try_from).collect()
    }
    None => bad_extension,
  }
}
