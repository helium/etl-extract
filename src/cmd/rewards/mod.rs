use crate::{cmd::Opts, Result};
use sqlx::PgPool;
use structopt::StructOpt;

mod account;
mod hex;
mod network;

#[derive(Debug, StructOpt)]
pub enum Cmd {
    Hex(hex::Cmd),
    Network(network::Cmd),
    Account(account::Cmd),
}

impl Cmd {
    pub async fn run(&self, pool: &PgPool, opts: Opts) -> Result {
        match self {
            Self::Hex(cmd) => cmd.run(pool, opts).await,
            Self::Network(cmd) => cmd.run(pool, opts).await,
            Self::Account(cmd) => cmd.run(pool, opts).await,
        }
    }
}
