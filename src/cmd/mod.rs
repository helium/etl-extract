use crate::Result;
use futures::stream::{BoxStream, TryStreamExt};
use std::path::PathBuf;
use structopt::StructOpt;

pub mod blocks;
// pub mod flow;
pub mod hotspots;
pub mod rewards;

/// Common options for most commands
#[derive(Debug, StructOpt)]
pub struct Opts {
    #[structopt(short = "e")]
    pub env: Option<PathBuf>,
}

pub fn print_json<T: ?Sized + serde::Serialize>(value: &T) -> Result {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

#[derive(Debug, Default)]
pub enum Format {
    #[default]
    Json,
    Csv,
}

impl Format {
    pub async fn output<'a, W, E>(
        &self,
        output: W,
        mut rows: BoxStream<'a, std::result::Result<E, sqlx::Error>>,
    ) -> Result
    where
        W: std::io::Write,
        E: serde::Serialize,
    {
        match self {
            Self::Json => {
                use serde::{ser::SerializeSeq, Serializer};
                let mut serializer = serde_json::Serializer::pretty(output);
                let mut entries = serializer.serialize_seq(None)?;
                while let Some(row) = rows.try_next().await? {
                    entries.serialize_element(&row)?;
                }
                entries.end()?;
            }
            Self::Csv => {
                let mut serializer = csv::Writer::from_writer(output);
                while let Some(row) = rows.try_next().await? {
                    serializer.serialize(&row)?;
                }
                serializer.flush()?;
            }
        }
        Ok(())
    }
}

impl std::str::FromStr for Format {
    type Err = crate::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        use anyhow::anyhow;
        match s.to_ascii_lowercase().as_str() {
            "csv" => Ok(Self::Csv),
            "json" => Ok(Self::Json),
            _ => Err(anyhow!("invalid format {s}")),
        }
    }
}

impl ToString for Format {
    fn to_string(&self) -> String {
        match self {
            Self::Csv => "csv".to_string(),
            Self::Json => "json".to_string(),
        }   
    }
}
