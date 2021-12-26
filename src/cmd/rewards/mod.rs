use crate::{cmd::Opts, Result};
use sqlx::PgPool;
use structopt::StructOpt;

mod hex;

#[derive(Debug, StructOpt)]
pub enum Cmd {
    Hex(hex::Cmd),
}

impl Cmd {
    pub async fn run(&self, pool: &PgPool, opts: Opts) -> Result {
        match self {
            Self::Hex(cmd) => cmd.run(pool, opts).await,
        }
    }
}
