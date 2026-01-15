//! Market value-based prediction strategy.
//!
//! Predicts match outcomes based on squad market values.
//! Theory: Higher market value correlates with team strength.

use crate::traits::{GoalExpectation, MatchContext, MatchProbabilities, PredictionStrategy};

/// Market value-based prediction strategy.
///
/// Uses log-scale comparison of squad market values to
/// calculate win probabilities.
#[derive(Debug, Clone)]
pub struct MarketValueStrategy {
    /// Base goal expectation for average teams
    pub base_goals: f64,
}

impl Default for MarketValueStrategy {
    fn default() -> Self {
        Self { base_goals: 1.3 }
    }
}

impl MarketValueStrategy {
    /// Create a new strategy with custom base goals.
    pub fn new(base_goals: f64) -> Self {
        Self { base_goals }
    }

    /// Calculate strength ratio from market values using log scale.
    /// Returns value between 0 and 1, where 0.5 means equal strength.
    fn value_ratio(&self, home_value: f64, away_value: f64) -> f64 {
        // Use log scale to prevent extreme ratios
        // Add 1 to handle zero values
        let home_log = (home_value + 1.0).ln();
        let away_log = (away_value + 1.0).ln();

        if home_log + away_log == 0.0 {
            0.5
        } else {
            home_log / (home_log + away_log)
        }
    }

    /// Calculate draw probability based on value difference.
    fn draw_probability(&self, ratio: f64) -> f64 {
        // Draw probability decreases as value difference increases
        let diff = (ratio - 0.5).abs();
        0.28 * (1.0 - diff * 2.0).max(0.0)
    }
}

impl PredictionStrategy for MarketValueStrategy {
    fn name(&self) -> &str {
        "Market Value"
    }

    fn predict_probabilities(&self, ctx: &MatchContext) -> MatchProbabilities {
        let ratio = self.value_ratio(
            ctx.home_team.market_value_millions,
            ctx.away_team.market_value_millions,
        );

        let draw_prob = if ctx.is_knockout {
            0.0
        } else {
            self.draw_probability(ratio)
        };

        let remaining = 1.0 - draw_prob;

        MatchProbabilities::new(ratio * remaining, draw_prob, (1.0 - ratio) * remaining)
    }

    fn predict_goals(&self, ctx: &MatchContext) -> GoalExpectation {
        let ratio = self.value_ratio(
            ctx.home_team.market_value_millions,
            ctx.away_team.market_value_millions,
        );

        // Scale goals based on relative strength
        GoalExpectation::new(
            self.base_goals * (0.5 + ratio),
            self.base_goals * (1.5 - ratio),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wc_core::{Confederation, Team, TeamId};

    fn create_team(id: u8, market_value: f64) -> Team {
        Team::new(TeamId(id), format!("Team {}", id), format!("T{:02}", id), Confederation::Uefa)
            .with_market_value(market_value)
    }

    #[test]
    fn test_equal_values() {
        let strategy = MarketValueStrategy::default();
        let ctx = MatchContext::new(
            create_team(0, 500.0),
            create_team(1, 500.0),
            false,
        );

        let probs = strategy.predict_probabilities(&ctx);

        // Equal values should give ~equal win probabilities
        assert!((probs.home_win - probs.away_win).abs() < 0.05);
    }

    #[test]
    fn test_higher_value_favored() {
        let strategy = MarketValueStrategy::default();
        let ctx = MatchContext::new(
            create_team(0, 1000.0), // Higher value
            create_team(1, 200.0),  // Lower value
            false,
        );

        let probs = strategy.predict_probabilities(&ctx);

        assert!(probs.home_win > probs.away_win);
    }

    #[test]
    fn test_zero_values_handled() {
        let strategy = MarketValueStrategy::default();
        let ctx = MatchContext::new(create_team(0, 0.0), create_team(1, 0.0), false);

        let probs = strategy.predict_probabilities(&ctx);

        // Should not panic and should give ~equal probabilities
        assert!((probs.home_win - probs.away_win).abs() < 0.1);
    }

    #[test]
    fn test_log_scale_prevents_extremes() {
        let strategy = MarketValueStrategy::default();
        // Huge difference in value
        let ctx = MatchContext::new(
            create_team(0, 2000.0),
            create_team(1, 50.0),
            false,
        );

        let probs = strategy.predict_probabilities(&ctx);

        // Even with huge difference, underdog should still have some chance
        assert!(probs.away_win > 0.1);
    }
}
