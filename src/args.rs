use std::path::PathBuf;

use clap::{Parser, ValueEnum, Subcommand, Args};

#[derive(Subcommand)]
pub enum TaggyCmd {
  /// Loads some .ics calendar and gives it a name.
  CalLoad{
    /// Path to .ics file
    path: PathBuf,
    /// Preferred name of calendar
    name: String,
  },

  /// Removes some .ics calendar.
  CalRm{
    /// Name of calendar
    name: String,
  },

  /// Shows current time.
  Now,

  /// Shows current timezone.
  Tz,

  /// Sets Timezone.
  TzSet{
    /// Timezone string expression, i.e. -4:00 means EDT.
    tz_expr: String,
  }
}

/// Taggytime mode
#[derive(Clone)]
#[derive(ValueEnum)]
pub enum Mode {
  /// enters interactive mode
  Interactive,
  /// Creates template environment
  Template,
}

/// Stores information parsed from commandline args.
#[derive(Parser)]
#[command(about = "TaggyTime: time management in one click")]
pub struct CliInfo {
  /// Path to TaggyTime environment
  pub envpath: PathBuf,

  // TaggyTime execution mode
  pub mode: Mode,

  // Interaction
  #[arg(short)]
  pub interact: bool,

  #[command(subcommand)]
  pub cmd: TaggyCmd,
}
