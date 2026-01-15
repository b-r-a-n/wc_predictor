//! Core trait for prediction strategies.

use rand::RngCore;
use serde::{Deserialize, Serialize};
use wc_core::{MatchResult, PenaltyResult, Team};

/// Context provided to prediction strategies for a match.
#[derive(Debug, Clone)]
pub struct MatchContext {
    /// Home/first team
    pub home_team: Team,
    /// Away/second team
    pub away_team: Team,
    /// Whether this is a knockout match (no draws allowed)
    pub is_knockout: bool,
    /// Importance factor (higher for later knockout rounds)
    pub round_importance: f64,
    /// Whether the match is at a neutral venue
    pub neutral_venue: bool,
}

impl MatchContext {
    /// Create a new match context.
    pub fn new(home_team: Team, away_team: Team, is_knockout: bool) -> Self {
        Self {
            home_team,
            away_team,
            is_knockout,
            round_importance: 1.0,
            neutral_venue: true, // World Cup uses neutral venues
        }
    }

    /// Set the round importance.
    pub fn with_importance(mut self, importance: f64) -> Self {
        self.round_importance = importance;
        self
    }
}

/// Outcome probabilities for a match.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MatchProbabilities {
    /// Probability of home/first team winning
    pub home_win: f64,
    /// Probability of a draw
    pub draw: f64,
    /// Probability of away/second team winning
    pub away_win: f64,
}

impl MatchProbabilities {
    /// Create new probabilities (will be normalized).
    pub fn new(home_win: f64, draw: f64, away_win: f64) -> Self {
        let total = home_win + draw + away_win;
        Self {
            home_win: home_win / total,
            draw: draw / total,
            away_win: away_win / total,
        }
    }

    /// Check if probabilities sum to 1.0 (within tolerance).
    pub fn is_valid(&self) -> bool {
        let sum = self.home_win + self.draw + self.away_win;
        (sum - 1.0).abs() < 0.0001
    }
}

/// Expected goals parameters for Poisson distribution.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GoalExpectation {
    /// Lambda (mean) for home team goals
    pub home_lambda: f64,
    /// Lambda (mean) for away team goals
    pub away_lambda: f64,
}

impl GoalExpectation {
    /// Create new goal expectations.
    pub fn new(home_lambda: f64, away_lambda: f64) -> Self {
        Self {
            home_lambda: home_lambda.max(0.1), // Minimum to avoid edge cases
            away_lambda: away_lambda.max(0.1),
        }
    }
}

/// Core trait for pluggable prediction algorithms.
///
/// Implementors provide match probability calculations and can
/// optionally override the match simulation logic.
pub trait PredictionStrategy: Send + Sync {
    /// Human-readable name for this strategy.
    fn name(&self) -> &str;

    /// Calculate match outcome probabilities.
    fn predict_probabilities(&self, ctx: &MatchContext) -> MatchProbabilities;

    /// Calculate expected goals (for goal-based simulation).
    fn predict_goals(&self, ctx: &MatchContext) -> GoalExpectation;

    /// Simulate a complete match result using the given RNG.
    ///
    /// The default implementation uses Poisson-distributed goals.
    fn simulate_match(&self, ctx: &MatchContext, rng: &mut dyn RngCore) -> MatchResult {
        let goals = self.predict_goals(ctx);

        // Sample goals from Poisson distributions
        let mut home_goals = sample_poisson(rng, goals.home_lambda);
        let mut away_goals = sample_poisson(rng, goals.away_lambda);

        let mut result = MatchResult::new(ctx.home_team.id, ctx.away_team.id, home_goals, away_goals);

        // Handle knockout draws with extra time and penalties
        if ctx.is_knockout && home_goals == away_goals {
            result.extra_time = true;

            // Extra time with reduced goal expectation (30% of normal)
            let et_home = sample_poisson(rng, goals.home_lambda * 0.3);
            let et_away = sample_poisson(rng, goals.away_lambda * 0.3);

            home_goals += et_home;
            away_goals += et_away;
            result.home_goals = home_goals;
            result.away_goals = away_goals;

            // If still tied, simulate penalties
            if home_goals == away_goals {
                result.penalties = Some(simulate_penalties(rng));
            }
        }

        result
    }
}

/// Generate a random f64 in [0, 1) from an RngCore.
fn gen_f64(rng: &mut dyn RngCore) -> f64 {
    // Use u64 for better precision
    let bits = rng.next_u64();
    // Convert to f64 in [0, 1)
    (bits >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
}

/// Sample from a Poisson distribution with the given lambda.
fn sample_poisson(rng: &mut dyn RngCore, lambda: f64) -> u8 {
    // Knuth algorithm for Poisson sampling
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
            // Cap at 15 goals (extremely rare in real matches)
            break;
        }
    }

    k
}

/// Simulate a penalty shootout.
fn simulate_penalties(rng: &mut dyn RngCore) -> PenaltyResult {
    const BASE_CONVERSION_RATE: f64 = 0.75;

    let mut home_score = 0u8;
    let mut away_score = 0u8;

    // First 5 rounds
    for round in 0..5 {
        // Home team shoots
        if gen_f64(rng) < BASE_CONVERSION_RATE {
            home_score += 1;
        }

        // Away team shoots
        if gen_f64(rng) < BASE_CONVERSION_RATE {
            away_score += 1;
        }

        // Early termination if one team can't catch up
        let remaining = (4 - round) as u8;
        if home_score > away_score + remaining {
            break;
        }
        if away_score > home_score + remaining {
            break;
        }
    }

    // Sudden death if tied after 5 rounds
    while home_score == away_score {
        let home_converts = gen_f64(rng) < BASE_CONVERSION_RATE;
        let away_converts = gen_f64(rng) < BASE_CONVERSION_RATE;

        if home_converts {
            home_score += 1;
        }
        if away_converts {
            away_score += 1;
        }

        // If both score or both miss, continue
        // If only one scores, they win
    }

    PenaltyResult {
        home_penalties: home_score,
        away_penalties: away_score,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use wc_core::{Confederation, TeamId};

    fn create_test_team(id: u8, elo: f64) -> Team {
        Team::new(TeamId(id), format!("Team {}", id), format!("T{:02}", id), Confederation::Uefa)
            .with_elo(elo)
    }

    struct TestStrategy;

    impl PredictionStrategy for TestStrategy {
        fn name(&self) -> &str {
            "Test"
        }

        fn predict_probabilities(&self, _ctx: &MatchContext) -> MatchProbabilities {
            MatchProbabilities::new(0.4, 0.3, 0.3)
        }

        fn predict_goals(&self, _ctx: &MatchContext) -> GoalExpectation {
            GoalExpectation::new(1.5, 1.2)
        }
    }

    #[test]
    fn test_poisson_sampling() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let samples: Vec<u8> = (0..1000).map(|_| sample_poisson(&mut rng, 1.5)).collect();

        let mean: f64 = samples.iter().map(|&x| x as f64).sum::<f64>() / 1000.0;
        // Mean should be close to lambda (1.5)
        assert!((mean - 1.5).abs() < 0.2);
    }

    #[test]
    fn test_match_simulation() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let strategy = TestStrategy;

        let ctx = MatchContext::new(create_test_team(0, 1800.0), create_test_team(1, 1700.0), false);

        let result = strategy.simulate_match(&ctx, &mut rng);

        assert_eq!(result.home_team, TeamId(0));
        assert_eq!(result.away_team, TeamId(1));
        assert!(!result.extra_time); // Group stage match
    }

    #[test]
    fn test_knockout_no_draw() {
        let mut rng = ChaCha8Rng::seed_from_u64(123);
        let strategy = TestStrategy;

        let ctx = MatchContext::new(create_test_team(0, 1800.0), create_test_team(1, 1700.0), true);

        // Run many simulations to ensure knockout matches always have a winner
        for _ in 0..100 {
            let result = strategy.simulate_match(&ctx, &mut rng);
            assert!(result.winner().is_some(), "Knockout match must have a winner");
        }
    }

    #[test]
    fn test_probabilities_valid() {
        let probs = MatchProbabilities::new(0.5, 0.3, 0.2);
        assert!(probs.is_valid());
    }
}
