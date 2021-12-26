use crate::{cmd::Opts, BlockSpan, Result};
use chrono::NaiveDate;
use futures::TryStreamExt;
use serde::{ser::SerializeSeq, Serializer};
use sqlx::postgres::PgPool;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Generates JSON output with rewards for each res8 hex that has hotspots.  
pub struct Cmd {
    // The day to run the report over (in UTC). The start time is at the
    // beginning midnight of the given date (00:00:00).
    date: NaiveDate,

    /// The number of days to include in the timespan. Days can be positive or
    /// negative.
    #[structopt(default_value = "-1")]
    days: i64,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct HexReward {
    hex: String,
    amount: f64,
    count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[sqlx(default)]
    avg: Option<f64>,
}

const HEXREWARDS_QUERY: &str = r#"
    with stats as (
        select r.gateway, sum(r.amount) as amount
        from gateway_inventory g
        left join rewards r on g.address = r.gateway
        where r.gateway is not null 
            and g.location_hex is not null
            and r.block between $1 and $2
        group by r.gateway
    )
    select 
        g.location_hex as hex, 
        (sum(s.amount) / 100000000)::float as amount, 
        count(s.gateway) as count
    from stats s
        left join gateway_inventory g on s.gateway = g.address
    group by g.location_hex;    
"#;

impl Cmd {
    pub async fn run(&self, pool: &PgPool, _opts: Opts) -> Result {
        let blockspan = BlockSpan::from_date(pool, self.date, self.days).await?;
        let mut rows = sqlx::query_as::<_, HexReward>(HEXREWARDS_QUERY)
            .bind(blockspan.low)
            .bind(blockspan.high)
            .fetch(pool);
        let mut serializer = serde_json::Serializer::pretty(std::io::stdout());
        let mut entries = serializer.serialize_seq(None)?;
        let abs_days = self.days.abs();
        while let Some(mut reward) = rows.try_next().await? {
            if abs_days > 1 {
                reward.avg = Some(reward.amount / abs_days as f64)
            }
            entries.serialize_element(&reward)?;
        }
        entries.end()?;
        Ok(())
    }
}
