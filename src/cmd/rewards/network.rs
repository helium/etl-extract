use crate::{cmd::Opts, BlockSpan, Result};
use chrono::NaiveDate;
use serde_json::json;
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
pub struct NetworkRewards {
    min: f64,
    max: f64,
    total: f64,
    median: f64,
    stddev: f64,
}

const REWARDS_QUERY: &str = r#"
    with reward_data as (
        select
            r.amount,
            r.gateway,
            r.time
        from rewards r
        where r. block between $1 and $2
    )
    select
        coalesce(min(d.amount) / 100000000, 0)::float as min,
        coalesce(max(d.amount) / 100000000, 0)::float as max,
        coalesce(sum(d.amount) / 100000000, 0)::float as total,
        coalesce(percentile_cont(0.5) within group (order by d.amount) / 100000000, 0)::float as median,
        coalesce(avg(d.amount) / 100000000, 0)::float as avg,
        coalesce(stddev(d.amount) / 100000000, 0)::float as stddev
    from (
        select
            sum(r.amount) as amount
        from reward_data r
        group by r.time
    ) d;
"#;

const HOTSPOTS_ONLINE: &str = r#"
    select count(*) from gateway_status g 
    where g.online = 'online';
"#;

const GET_VAR: &str = r#"
    select value::float from vars_inventory where name = $1;
"#;

impl Cmd {
    pub async fn run(&self, pool: &PgPool, _opts: Opts) -> Result {
        let blockspan = BlockSpan::from_date(pool, self.date, self.days).await?;
        let rewards = sqlx::query_as::<_, NetworkRewards>(REWARDS_QUERY)
            .bind(blockspan.low)
            .bind(blockspan.high)
            .fetch_one(pool)
            .await?;

        let (hotspots_online,): (i64,) = sqlx::query_as(HOTSPOTS_ONLINE).fetch_one(pool).await?;
        let (securities_percent,): (f64,) = sqlx::query_as(GET_VAR)
            .bind("securities_percent")
            .fetch_one(pool)
            .await?;
        let (consensus_percent,): (f64,) = sqlx::query_as(GET_VAR)
            .bind("consensus_percent")
            .fetch_one(pool)
            .await?;
        let hotspot_avg_rewards = (rewards.total / self.days.abs() as f64)
            * (1.0 - consensus_percent - securities_percent)
            / hotspots_online as f64;

        let summary = json!({
            "securities_percent": securities_percent,
            "consensus_percent": consensus_percent,
            "hotspots_online": hotspots_online,
            "hotspot_avg_rewards": hotspot_avg_rewards,
            "rewards": rewards,
        });
        println!("{}", serde_json::to_string_pretty(&summary)?);
        Ok(())
    }
}
