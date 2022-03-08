use crate::{cmd::Opts, BlockSpan, Result};
use chrono::NaiveDate;
use futures::TryStreamExt;
use h3ron::{H3Cell, ToCoordinate};
use serde::{ser::SerializeSeq, Serializer};
use sqlx::postgres::PgPool;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Generates JSON output with all hotspots
pub struct Cmd {
    /// The day to run the report over (in UTC). The start time is at the
    /// beginning midnight of the given date (00:00:00).
    date: NaiveDate,

    /// The number of days to include in the timespan. Days can be positive or
    /// negative.
    #[structopt(default_value = "-1")]
    days: i64,
}

#[derive(sqlx::Type, Debug, serde::Serialize, serde::Deserialize)]
#[sqlx(type_name = "gateway_status_online", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum HotspotStatus {
    Online,
    Offline,
}

#[derive(sqlx::Type, Debug, serde::Serialize, serde::Deserialize)]
#[sqlx(type_name = "gateway_mode", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum HotspotMode {
    Dataonly,
    Full,
    Light,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct Hotspot {
    address: String,
    mode: HotspotMode,
    owner: String,
    location: Option<String>,
    name: String,
    online: HotspotStatus,
    #[sqlx(default)]
    lat: Option<f64>,
    #[sqlx(default)]
    lng: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    short_street: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    short_city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    short_country: Option<String>,
}

const HOTSPOTS_QUERY: &str = r#"
    select
        g.address,
        g.mode,
        g.owner,
        g.location,
        g.name,
        s.online,
        l.short_street,
        l.short_city,
        l.short_state,
        l.short_country
    from gateway_inventory g
    left join locations l on g.location = l.location
    left join gateway_status s on s.address = g.address
    order by g.first_block desc, g.address;
"#;

impl Cmd {
    pub async fn run(&self, pool: &PgPool, _opts: Opts) -> Result {
        let blockspan = BlockSpan::from_date(pool, self.date, self.days).await?;
        let mut rows = sqlx::query_as::<_, Hotspot>(HOTSPOTS_QUERY)
            .bind(blockspan.low)
            .bind(blockspan.high)
            .fetch(pool);
        let mut serializer = serde_json::Serializer::pretty(std::io::stdout());
        let mut entries = serializer.serialize_seq(None)?;
        while let Some(mut hotspot) = rows.try_next().await? {
            if let Some(location) = &hotspot.location {
                let (lng, lat) = H3Cell::from_str(location)?.to_coordinate().x_y();
                hotspot.lng = Some(lng);
                hotspot.lat = Some(lat);
            }
            entries.serialize_element(&hotspot)?;
        }
        entries.end()?;
        Ok(())
    }
}
