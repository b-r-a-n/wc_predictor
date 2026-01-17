//! Prediction strategies for World Cup simulation.
//!
//! This crate provides the `PredictionStrategy` trait and several implementations:
//! - ELO-based prediction
//! - Market value-based prediction
//! - FIFA ranking-based prediction
//! - Form-based prediction (Sofascore)
//! - Composite strategies

pub mod composite;
pub mod elo;
pub mod fifa_ranking;
pub mod form;
pub mod market_value;
pub mod traits;

pub use composite::CompositeStrategy;
pub use elo::EloStrategy;
pub use fifa_ranking::FifaRankingStrategy;
pub use form::FormStrategy;
pub use market_value::MarketValueStrategy;
pub use traits::{GoalExpectation, MatchContext, MatchProbabilities, PredictionStrategy};
