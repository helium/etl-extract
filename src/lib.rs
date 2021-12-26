pub type Result<T = ()> = anyhow::Result<T>;
pub type Error = anyhow::Error;
pub use anyhow::{anyhow, bail};

pub mod cmd;

mod blockspan;
mod timespan;

pub use blockspan::BlockSpan;
pub use timespan::TimeSpan;
