//! CLI argument definitions using clap.

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "wc")]
#[command(author, version, about = "World Cup 2026 Prediction Simulator")]
#[command(propagate_version = true)]
pub struct Cli {
    /// Output format (table or json)
    #[arg(long, global = true, default_value = "table")]
    pub format: OutputFormat,

    /// Path to tournament data JSON file (default: embedded data)
    #[arg(long, global = true)]
    pub data: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(ValueEnum, Clone, Default, Copy)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run full tournament simulation
    Simulate(SimulateArgs),

    /// Predict a single match between two teams
    Match(MatchArgs),

    /// Show team information
    Team(TeamArgs),

    /// List all teams
    Teams(TeamsArgs),
}

#[derive(Parser)]
pub struct SimulateArgs {
    /// Number of simulations to run
    #[arg(short = 'n', long, default_value = "10000")]
    pub iterations: u32,

    /// Prediction strategy to use
    #[arg(short, long, default_value = "elo")]
    pub strategy: StrategyChoice,

    /// Random seed for reproducibility
    #[arg(long)]
    pub seed: Option<u64>,

    /// Number of parallel threads (default: auto-detect)
    #[arg(long)]
    pub threads: Option<usize>,

    /// Show top N teams (default: 10)
    #[arg(long, default_value = "10")]
    pub top: usize,
}

#[derive(ValueEnum, Clone, Default, Copy)]
pub enum StrategyChoice {
    /// ELO rating based prediction
    #[default]
    Elo,
    /// FIFA world ranking based prediction
    Fifa,
    /// Squad market value based prediction
    Market,
    /// Recent form based prediction (Sofascore)
    Form,
    /// Composite strategy (weighted combination)
    Composite,
}

#[derive(Parser)]
pub struct MatchArgs {
    /// First team (name or code, e.g., "Brazil" or "BRA")
    pub team_a: String,

    /// Second team (name or code)
    pub team_b: String,

    /// Treat as knockout match (no draws)
    #[arg(short, long)]
    pub knockout: bool,

    /// Prediction strategy
    #[arg(short, long, default_value = "elo")]
    pub strategy: StrategyChoice,

    /// Simulate N matches and show distribution
    #[arg(long)]
    pub simulate: Option<u32>,

    /// Random seed (for --simulate)
    #[arg(long)]
    pub seed: Option<u64>,
}

#[derive(Parser)]
pub struct TeamArgs {
    /// Team name or code
    pub team: String,
}

#[derive(Parser)]
pub struct TeamsArgs {
    /// Sort by field
    #[arg(long, default_value = "name")]
    pub sort: TeamSortField,

    /// Reverse sort order
    #[arg(long)]
    pub reverse: bool,

    /// Filter by confederation (UEFA, CONMEBOL, CONCACAF, CAF, AFC, OFC)
    #[arg(long)]
    pub confederation: Option<String>,
}

#[derive(ValueEnum, Clone, Default, Copy)]
pub enum TeamSortField {
    #[default]
    Name,
    Elo,
    FifaRank,
    MarketValue,
}
