//! Monte Carlo simulation engine for World Cup prediction.
//!
//! This crate provides:
//! - `SimulationEngine` - runs a single tournament simulation
//! - `SimulationRunner` - runs multiple simulations in parallel
//! - `AggregatedResults` - statistics from multiple simulations

pub mod aggregator;
pub mod engine;
pub mod runner;

pub use aggregator::{AggregatedResults, TeamStatistics};
pub use engine::SimulationEngine;
pub use runner::{SimulationConfig, SimulationRunner};
