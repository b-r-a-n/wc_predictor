//! Monte Carlo simulation engine for World Cup prediction.
//!
//! This crate provides:
//! - `SimulationEngine` - runs a single tournament simulation
//! - `SimulationRunner` - runs multiple simulations in parallel
//! - `AggregatedResults` - statistics from multiple simulations
//! - `PathStatistics` - tournament path tracking through knockout stages

pub mod aggregator;
pub mod engine;
pub mod optimal_bracket;
pub mod path_tracker;
pub mod runner;

pub use aggregator::{AggregatedResults, TeamStatistics};
pub use engine::SimulationEngine;
pub use optimal_bracket::compute_optimal_bracket;
pub use path_tracker::{BracketSlotStats, BracketSlotWinStats, MostLikelyBracket, MostLikelyBracketSlot, OptimalBracket, OptimalR32Match, PathStatistics, RoundMatchups, SlotOpponentStats};
pub use runner::{SimulationConfig, SimulationRunner};
