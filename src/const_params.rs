//! Dedicated to store compile-time-known constants that influences
//! design choices.

pub const ICS_ASSUME_TRANSP_ALWAYS_AFTER_SUMMARY: bool = true;
pub const ICS_DEFAULT_TIME_IN_DAY: &str = "235900";
pub const MAX_WORKLOAD: u32 = 60_000;
pub const HANDLE_WKST: bool = false;
pub const PARSE_DT_LITERAL_TZID: bool = false;
pub const TAGGYENV_PATH: &str = "/usr/local/taggytime/env.json";