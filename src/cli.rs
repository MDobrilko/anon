use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    #[arg(long, short)]
    pub config: PathBuf,
    #[command(subcommand)]
    pub command: Option<Command>,
}

impl Args {
    pub fn parse() -> Self {
        Parser::parse()
    }
}

#[derive(Debug, Subcommand, Clone)]
pub enum Command {
    Setup,
    Run,
}
