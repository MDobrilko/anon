use crate::{
    cli::{Args, Command},
    config::Config,
};

mod bot;
mod cli;
mod config;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = Config::open(&args.config)?;

    match args.command {
        Some(Command::Setup) => todo!(),
        Some(Command::Run) | None => bot::run(config)?,
    }

    Ok(())
}
