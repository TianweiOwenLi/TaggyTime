use std::path::PathBuf;

use clap::{Parser, ValueEnum};

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
}
