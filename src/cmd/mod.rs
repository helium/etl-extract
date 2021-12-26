pub mod blocks;
pub mod rewards;

use crate::Result;
use structopt::StructOpt;

/// Common options for most commands
#[derive(Debug, StructOpt)]
pub struct Opts {}

pub fn print_json<T: ?Sized + serde::Serialize>(value: &T) -> Result {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
