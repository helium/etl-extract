use etl_exporter::{
    cmd::{blocks, rewards, Opts},
    Result,
};
use sqlx::postgres::PgPool;
use std::process;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(global_setting = structopt::clap::AppSettings::AllowNegativeNumbers)]
pub struct Cli {
    #[structopt(flatten)]
    cmd: Cmd,
    #[structopt(flatten)]
    opts: Opts,
}

#[derive(Debug, StructOpt)]
pub enum Cmd {
    Blocks(blocks::Cmd),
    Rewards(rewards::Cmd),
}

#[tokio::main]
async fn main() -> Result {
    let cli = Cli::from_args();
    if let Err(e) = run(cli).await {
        eprintln!("error: {:?}", e);
        process::exit(1);
    }

    Ok(())
}

async fn run(cli: Cli) -> Result {
    let pool = PgPool::connect(&dotenv::var("DATABASE_URL")?).await?;

    match cli.cmd {
        Cmd::Blocks(cmd) => cmd.run(&pool, cli.opts).await,
        Cmd::Rewards(cmd) => cmd.run(&pool, cli.opts).await,
    }
}