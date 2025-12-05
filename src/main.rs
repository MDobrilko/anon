#![allow(deprecated)]

use crate::{
    cli::{Args, Command},
    config::Config,
};

mod bot;
mod cli;
mod config;
mod log;
mod state;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = Config::open(&args.config)?;

    let _global_logger = log::init(&config)?;

    match args.command {
        Some(Command::Setup) => bot::setup(config)?,
        Some(Command::Run) | None => bot::start(config)?,
    }

    Ok(())
}
