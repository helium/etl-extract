use crate::{timespan::ToDateTimeUtc, Result, TimeSpan};
use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct BlockSpan {
    pub low: i64,
    pub high: i64,
}

const BLOCKSPAN_QUERY: &str = r#"
    with max as (
        select height from blocks where timestamp <= $1 order by timestamp desc limit 1
    ),
    min as (
        select height from blocks where timestamp >= $2 order by timestamp limit 1
    )
    select (select height from max) as high, (select height from min) as low            
"#;

impl BlockSpan {
    pub async fn from_date<S: ToDateTimeUtc>(pool: &PgPool, date: S, days: i64) -> Result<Self> {
        let timespan = TimeSpan::new(date, days);
        Self::for_timespan(pool, &timespan).await
    }

    pub async fn for_timespan(pool: &PgPool, timespan: &TimeSpan) -> Result<Self> {
        let span: BlockSpan = sqlx::query_as(BLOCKSPAN_QUERY)
            .bind(timespan.high)
            .bind(timespan.low)
            .fetch_one(pool)
            .await?;
        Ok(span)
    }
}
