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
/// Gets the balance of the given account at the given date
pub struct Cmd {
    /// The wallet address to look up the balance for
    account: String,

    /// One or more end dates (exclusive) to run the report over (in UTC). The
    /// end time is at the beginning midnight of the given date (00:00:00).
    end: Vec<NaiveDate>,

    #[structopt(long, default_value)]
    format: Format,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct Balance {
    #[sqlx(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    date: Option<NaiveDate>,
    block: i64,
    dc: i64,
    hnt: f64,
    mobile: f64,
    iot: f64,
    hst: f64,
    staked_hnt: f64,
}

const BALANCE_QUERY: &str = r#"
    select 
        $2 as block, 
        greatest(0, max(balance))::float8 / 100000000 as hnt,
        greatest(0, max(mobile_balance))::float8 / 100000000 as mobile,
        greatest(0, max(security_balance))::float8 / 100000000 as hst,
        greatest(0, max(iot_balance))::float8 / 100000000 as iot,
        greatest(0, max(dc_balance)) as dc,
        greatest(0, max(staked_balance))::float8 / 100000000 as staked_hnt
    from accounts
    where 
        address = $1 and
        block > $2 and 
        block <= $3
"#;

impl Cmd {
    pub async fn run(&self, pool: &PgPool, _opts: Opts) -> Result {
        let supplies = stream::iter(self.end.clone())
            .map(|end| Ok(fetch_balance(pool, &self.account, end)))
            .try_buffered(10)
            .boxed();

        self.format.output(std::io::stdout(), supplies).await?;
        Ok(())
    }
}

async fn fetch_balance(
    pool: &PgPool,
    account: &str,
    end: NaiveDate,
) -> StdResult<Balance, sqlx::Error> {
    let start = end - chrono::Duration::days(1);
    let blockspan = BlockSpan::for_date_range(pool, start, end).await?;
    let mut balance: Balance = sqlx::query_as::<_, Balance>(BALANCE_QUERY)
        .bind(account)
        .bind(blockspan.low)
        .bind(blockspan.high)
        .fetch_one(pool)
        .await?;
    balance.date = Some(end);
    Ok(balance)
}
