//! Monte Carlo simulation engine for World Cup prediction.
//!
//! This crate provides:
//! - `SimulationEngine` - runs a single tournament simulation
//! - `SimulationRunner` - runs multiple simulations in parallel
//! - `AggregatedResults` - statistics from multiple simulations
//! - `PathStatistics` - tournament path tracking through knockout stages

pub mod aggregator;
pub mod engine;
pub mod path_tracker;
pub mod runner;

pub use aggregator::{AggregatedResults, TeamStatistics};
pub use engine::SimulationEngine;
pub use path_tracker::{BracketSlotStats, BracketSlotWinStats, MostLikelyBracket, MostLikelyBracketSlot, PathStatistics, RoundMatchups, SlotOpponentStats};
pub use runner::{SimulationConfig, SimulationRunner};
