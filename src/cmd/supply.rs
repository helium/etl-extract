use crate::{
    cmd::{Format, Opts},
    BlockSpan, Result,
};
use chrono::NaiveDate;
use futures::stream::{self, StreamExt, TryStreamExt};
use sqlx::postgres::PgPool;
use std::result::Result as StdResult;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Gets the running supply at a given date.
pub struct Cmd {
    /// One or more end dates (exclusive) to run the report over (in UTC). The
    /// end time is at the beginning midnight of the given date (00:00:00).
    end: Vec<NaiveDate>,

    #[structopt(long, default_value)]
    format: Format,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct Supply {
    #[sqlx(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    date: Option<NaiveDate>,
    block: i64,
    hnt: f64,
}

const SUPPLY_QUERY: &str = r#"
    with balances as (
        select address, max(balance) as balance
        from accounts
        where block <= $1 and balance > 0
        group by address
    ) 
    select $1 as block, greatest(0, sum(balance)::float8) / 100000000 as hnt from balances;
"#;

impl Cmd {
    pub async fn run(&self, pool: &PgPool, _opts: Opts) -> Result {
        let supplies = stream::iter(self.end.clone())
            .map(|end| Ok(fetch_supply(pool, end)))
            .try_buffered(10)
            .boxed();

        self.format.output(std::io::stdout(), supplies).await?;
        Ok(())
    }
}

async fn fetch_supply(pool: &PgPool, end: NaiveDate) -> StdResult<Supply, sqlx::Error> {
    let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
    let blockspan = BlockSpan::for_date_range(pool, epoch, end).await?;
    let mut supply: Supply = sqlx::query_as::<_, Supply>(SUPPLY_QUERY)
        .bind(blockspan.high)
        .fetch_one(pool)
        .await?;
    supply.date = Some(end);
    Ok(supply)
}
