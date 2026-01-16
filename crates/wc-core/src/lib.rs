//! Core domain types for World Cup 2026 simulation.
//!
//! This crate provides the fundamental types used throughout the simulation:
//! - Teams and confederations
//! - Match results and outcomes
//! - Group stage structure and standings
//! - Tournament configuration and results

pub mod bracket;
pub mod group;
pub mod knockout;
pub mod match_result;
pub mod team;
pub mod tiebreaker;
pub mod tournament;

pub use group::{Group, GroupId, GroupResult};
pub use knockout::{KnockoutBracket, KnockoutRound};
pub use match_result::{MatchOutcome, MatchResult, PenaltyResult};
pub use team::{Confederation, Team, TeamId};
pub use tiebreaker::GroupStanding;
pub use tournament::{Tournament, TournamentResult};
