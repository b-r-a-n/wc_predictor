//! Composite strategy combining multiple prediction methods.

use rand::RngCore;

use crate::traits::{GoalExpectation, MatchContext, MatchProbabilities, PredictionStrategy};
use wc_core::MatchResult;

/// A weighted combination of multiple prediction strategies.
///
/// Combines probabilities and goal expectations from multiple
/// strategies using configurable weights.
pub struct CompositeStrategy {
    name: String,
    strategies: Vec<(Box<dyn PredictionStrategy>, f64)>,
}

impl CompositeStrategy {
    /// Create a new composite strategy with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            strategies: Vec::new(),
        }
    }

    /// Add a strategy with the given weight.
    pub fn add_strategy<S: PredictionStrategy + 'static>(mut self, strategy: S, weight: f64) -> Self {
        self.strategies.push((Box::new(strategy), weight));
        self
    }

    /// Get normalized weights that sum to 1.
    fn normalized_weights(&self) -> Vec<f64> {
        let total: f64 = self.strategies.iter().map(|(_, w)| w).sum();
        if total == 0.0 {
            vec![1.0 / self.strategies.len() as f64; self.strategies.len()]
        } else {
            self.strategies.iter().map(|(_, w)| w / total).collect()
        }
    }

    /// Get the number of strategies in this composite.
    pub fn num_strategies(&self) -> usize {
        self.strategies.len()
    }

    /// Check if this composite has any strategies.
    pub fn is_empty(&self) -> bool {
        self.strategies.is_empty()
    }
}

impl PredictionStrategy for CompositeStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    fn predict_probabilities(&self, ctx: &MatchContext) -> MatchProbabilities {
        if self.strategies.is_empty() {
            return MatchProbabilities::new(0.33, 0.34, 0.33);
        }

        let weights = self.normalized_weights();

        let mut home_win = 0.0;
        let mut draw = 0.0;
        let mut away_win = 0.0;

        for ((strategy, _), weight) in self.strategies.iter().zip(weights.iter()) {
            let probs = strategy.predict_probabilities(ctx);
            home_win += probs.home_win * weight;
            draw += probs.draw * weight;
            away_win += probs.away_win * weight;
        }

        MatchProbabilities::new(home_win, draw, away_win)
    }

    fn predict_goals(&self, ctx: &MatchContext) -> GoalExpectation {
        if self.strategies.is_empty() {
            return GoalExpectation::new(1.3, 1.3);
        }

        let weights = self.normalized_weights();

        let mut home_lambda = 0.0;
        let mut away_lambda = 0.0;

        for ((strategy, _), weight) in self.strategies.iter().zip(weights.iter()) {
            let goals = strategy.predict_goals(ctx);
            home_lambda += goals.home_lambda * weight;
            away_lambda += goals.away_lambda * weight;
        }

        GoalExpectation::new(home_lambda, away_lambda)
    }

    fn simulate_match(&self, ctx: &MatchContext, rng: &mut dyn RngCore) -> MatchResult {
        // Use default implementation from trait
        let goals = self.predict_goals(ctx);

        // Inline the simulation logic to avoid calling default impl
        let mut home_goals = sample_poisson(rng, goals.home_lambda);
        let mut away_goals = sample_poisson(rng, goals.away_lambda);

        let mut result = MatchResult::new(ctx.home_team.id, ctx.away_team.id, home_goals, away_goals);

        if ctx.is_knockout && home_goals == away_goals {
            result.extra_time = true;

            let et_home = sample_poisson(rng, goals.home_lambda * 0.3);
            let et_away = sample_poisson(rng, goals.away_lambda * 0.3);

            home_goals += et_home;
            away_goals += et_away;
            result.home_goals = home_goals;
            result.away_goals = away_goals;

            if home_goals == away_goals {
                result.penalties = Some(simulate_penalties(rng));
            }
        }

        result
    }
}

/// Generate a random f64 in [0, 1) from an RngCore.
fn gen_f64(rng: &mut dyn RngCore) -> f64 {
    let bits = rng.next_u64();
    (bits >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
}

/// Sample from Poisson distribution.
fn sample_poisson(rng: &mut dyn RngCore, lambda: f64) -> u8 {
    let l = (-lambda).exp();
    let mut k = 0u8;
    let mut p = 1.0;

    loop {
        p *= gen_f64(rng);
        if p <= l {
            break;
        }
        k = k.saturating_add(1);
        if k >= 15 {
            break;
        }
    }
    k
}

/// Simulate penalties.
fn simulate_penalties(rng: &mut dyn RngCore) -> wc_core::PenaltyResult {
    const RATE: f64 = 0.75;
    let mut home = 0u8;
    let mut away = 0u8;

    for round in 0..5 {
        if gen_f64(rng) < RATE { home += 1; }
        if gen_f64(rng) < RATE { away += 1; }

        let remaining = (4 - round) as u8;
        if home > away + remaining || away > home + remaining {
            break;
        }
    }

    while home == away {
        let h = gen_f64(rng) < RATE;
        let a = gen_f64(rng) < RATE;
        if h { home += 1; }
        if a { away += 1; }
    }

    wc_core::PenaltyResult {
        home_penalties: home,
        away_penalties: away,
    }
}

// Implement Send + Sync manually since we use trait objects
unsafe impl Send for CompositeStrategy {}
unsafe impl Sync for CompositeStrategy {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EloStrategy, MarketValueStrategy};
    use wc_core::{Confederation, Team, TeamId};

    fn create_team(id: u8, elo: f64, market_value: f64) -> Team {
        Team::new(TeamId(id), format!("Team {}", id), format!("T{:02}", id), Confederation::Uefa)
            .with_elo(elo)
            .with_market_value(market_value)
    }

    #[test]
    fn test_composite_combines_strategies() {
        let composite = CompositeStrategy::new("Test Composite")
            .add_strategy(EloStrategy::default(), 0.6)
            .add_strategy(MarketValueStrategy::default(), 0.4);

        let ctx = MatchContext::new(
            create_team(0, 2000.0, 1000.0),
            create_team(1, 1700.0, 500.0),
            false,
        );

        let probs = composite.predict_probabilities(&ctx);

        // Should give reasonable probabilities
        assert!(probs.is_valid());
        assert!(probs.home_win > probs.away_win); // Home team is stronger in both metrics
    }

    #[test]
    fn test_empty_composite() {
        let composite = CompositeStrategy::new("Empty");

        let ctx = MatchContext::new(
            create_team(0, 1800.0, 500.0),
            create_team(1, 1800.0, 500.0),
            false,
        );

        let probs = composite.predict_probabilities(&ctx);

        // Should give neutral probabilities
        assert!(probs.is_valid());
    }

    #[test]
    fn test_single_strategy_composite() {
        let composite = CompositeStrategy::new("Single")
            .add_strategy(EloStrategy::default(), 1.0);

        let ctx = MatchContext::new(
            create_team(0, 2000.0, 0.0),
            create_team(1, 1600.0, 0.0),
            false,
        );

        let composite_probs = composite.predict_probabilities(&ctx);
        let elo_probs = EloStrategy::default().predict_probabilities(&ctx);

        // Should match the single strategy exactly
        assert!((composite_probs.home_win - elo_probs.home_win).abs() < 0.001);
    }
}
