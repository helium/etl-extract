use crate::{cmd::Opts, Result};
use sqlx::PgPool;
use structopt::StructOpt;

mod network;

#[derive(Debug, StructOpt)]
pub enum Cmd {
    Network(network::Cmd),
}

impl Cmd {
    pub async fn run(&self, pool: &PgPool, opts: Opts) -> Result {
        match self {
            Self::Network(cmd) => cmd.run(pool, opts).await,
        }
    }
}
