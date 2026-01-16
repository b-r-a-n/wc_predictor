//! World Cup 2026 Prediction Simulator CLI.

mod cli;
mod commands;
mod data;
mod error;
mod output;

use clap::Parser;

use cli::{Cli, Commands};
use data::TournamentData;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> error::Result<()> {
    let cli = Cli::parse();

    // Load tournament data
    let tournament_data = TournamentData::load(cli.data.as_deref())?;
    let tournament = tournament_data.into_tournament()?;

    // Execute command
    match &cli.command {
        Commands::Simulate(args) => commands::run_simulate(args, &tournament, cli.format),
        Commands::Match(args) => commands::run_match(args, &tournament, cli.format),
        Commands::Team(args) => commands::run_team(args, &tournament, cli.format),
        Commands::Teams(args) => commands::run_teams(args, &tournament, cli.format),
    }
}
