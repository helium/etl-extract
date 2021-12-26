use crate::{
    cmd::{print_json, Opts},
    BlockSpan, Result, TimeSpan,
};
use chrono::NaiveDate;
use serde_json::json;
use sqlx::postgres::PgPool;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Look up the block range for a given date in UTC. If no date is specified the
/// current UTC date is used.
pub struct Cmd {
    /// The day to start the timespan at. The start time of the date is at
    /// midnight UTC.
    date: NaiveDate,

    /// The number of days to include in the timespan. Days can be positive or
    /// negative.
    #[structopt(default_value = "1")]
    days: i64,
}

impl Cmd {
    pub async fn run(&self, pool: &PgPool, _opts: Opts) -> Result {
        let timespan = TimeSpan::new(self.date, self.days);
        let blockspan = BlockSpan::for_timespan(pool, &timespan).await?;
        print_blockspan(&timespan, &blockspan)
    }
}

fn print_blockspan(timespan: &TimeSpan, blockspan: &BlockSpan) -> Result {
    let json = json!({
        "blockspan": blockspan,
        "timespan": timespan,
    });
    print_json(&json)
}
