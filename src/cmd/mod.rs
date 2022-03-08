pub mod blocks;
pub mod hotspots;
pub mod rewards;

use crate::Result;
use structopt::StructOpt;

use std::path::PathBuf;

/// Common options for most commands
#[derive(Debug, StructOpt)]
pub struct Opts {
    #[structopt(short = "e")]
    pub env: Option<PathBuf>,
}

pub fn print_json<T: ?Sized + serde::Serialize>(value: &T) -> Result {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
