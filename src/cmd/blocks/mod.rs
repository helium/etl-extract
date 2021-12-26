use crate::{cmd::Opts, Result};
use sqlx::PgPool;
use structopt::StructOpt;

mod span;

#[derive(Debug, StructOpt)]
pub enum Cmd {
    Span(span::Cmd),
}

impl Cmd {
    pub async fn run(&self, pool: &PgPool, opts: Opts) -> Result {
        match self {
            Self::Span(cmd) => cmd.run(pool, opts).await,
        }
    }
}
