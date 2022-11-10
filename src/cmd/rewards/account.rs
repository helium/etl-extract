use crate::{cmd::Opts, BlockSpan, Error, Result};
use anyhow::anyhow;
use chrono::{DateTime, NaiveDate, Utc};
use futures::stream::BoxStream;
use futures::TryStreamExt;
use serde::{ser::SerializeSeq, Serializer};
use sqlx::postgres::PgPool;
use structopt::StructOpt;

#[derive(Debug)]
pub enum Format {
    Json,
    Csv,
}

impl Format {
    async fn output<'a, W: std::io::Write>(
        &self,
        output: W,
        mut rows: BoxStream<'a, std::result::Result<ValidatorReward, sqlx::Error>>,
    ) -> Result {
        match self {
            Self::Json => {
                let mut serializer = serde_json::Serializer::pretty(output);
                let mut entries = serializer.serialize_seq(None)?;
                while let Some(reward) = rows.try_next().await? {
                    entries.serialize_element(&reward)?;
                }
                entries.end()?;
            }
            Self::Csv => {
                let mut serializer = csv::Writer::from_writer(output);
                while let Some(reward) = rows.try_next().await? {
                    serializer.serialize(&reward)?;
                }
                serializer.flush()?;
            }
        }
        Ok(())
    }
}

impl std::str::FromStr for Format {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "csv" => Ok(Self::Csv),
            "json" => Ok(Self::Json),
            _ => Err(anyhow!("invalid format {s}")),
        }
    }
}

#[derive(Debug, StructOpt)]
/// Generates CSV output with rewards for all reward entries for validators
/// owned by a given wallet.  
pub struct Cmd {
    /// The wallet address to look up validators for
    account: String,

    /// The start day (inclusive) to run the report over (in UTC). The start
    /// time is at the beginning midnight of the given date (00:00:00).
    start: NaiveDate,

    /// The end day (exclusive) to run the report over (in UTC). The end time is
    /// at the beginning midnight of the given date (00:00:00).
    end: NaiveDate,

    #[structopt(long, default_value = "json")]
    format: Format,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct ValidatorReward {
    block: i64,
    timestamp: DateTime<Utc>,
    reward_type: String,
    transaction_hash: String,
    validator: Option<String>,
    hnt: f64,
    usd_oracle_price: f64,
    usd_amount: f64,
}

const VALIDATOR_REWRDS_QUERY: &str = r#"
    select 
        t.block,
        to_timestamp(t.time) as timestamp,
        case when gateway = '1Wh4bh' then 'securities' else 'validator' end as reward_type,
        t.transaction_hash,
        (case when gateway = '1Wh4bh' then null else gateway end) as validator,
        amount::float8 / 100000000 as hnt,
        o.price::float8 / 100000000 as usd_oracle_price,
        (amount::float8 / 100000000) * (o.price::float8 / 100000000) as usd_amount
    from rewards t
    left join oracle_prices o on o.block = (select max(o2.block) from oracle_prices o2 where o2.block <= t.block)
    where t.block between $1 and $2
    and account = $3
    order by t.block asc;
"#;

impl Cmd {
    pub async fn run(&self, pool: &PgPool, _opts: Opts) -> Result {
        let blockspan = BlockSpan::for_date_range(pool, self.start, self.end).await?;
        let rows = sqlx::query_as::<_, ValidatorReward>(VALIDATOR_REWRDS_QUERY)
            .bind(blockspan.low)
            .bind(blockspan.high)
            .bind(&self.account)
            .fetch(pool);
        self.format.output(std::io::stdout(), rows).await?;
        Ok(())
    }
}
