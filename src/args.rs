use clap::Parser;

use crate::taggy_cmd::TaggyCmd;

/// Stores information parsed from commandline args.
#[derive(Parser)]
#[command(about = "TaggyTime: time management in one click")]
pub struct CliInfo {

  /// Interactive mode
  #[arg(short, long)]
  pub interactive: bool,

  #[command(subcommand)]
  pub cmd: TaggyCmd,
}
