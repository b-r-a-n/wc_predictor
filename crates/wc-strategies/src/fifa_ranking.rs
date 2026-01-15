//! FIFA ranking-based prediction strategy.
//!
//! Uses official FIFA world rankings to predict outcomes.

use crate::traits::{GoalExpectation, MatchContext, MatchProbabilities, PredictionStrategy};

/// FIFA ranking-based prediction strategy.
///
/// Converts FIFA rankings to win probabilities.
/// Lower ranking (e.g., 1) means stronger team.
#[derive(Debug, Clone)]
pub struct FifaRankingStrategy {
    /// Base goal expectation for average teams
    pub base_goals: f64,
    /// Maximum ranking to consider (teams ranked higher get same as max)
    pub max_ranking: u16,
}

impl Default for FifaRankingStrategy {
    fn default() -> Self {
        Self {
            base_goals: 1.3,
            max_ranking: 100, // Cap at 100 for normalization
        }
    }
}

impl FifaRankingStrategy {
    /// Create a new strategy with custom parameters.
    pub fn new(base_goals: f64, max_ranking: u16) -> Self {
        Self {
            base_goals,
            max_ranking,
        }
    }

    /// Convert ranking to strength score (0-1, higher is better).
    fn ranking_to_strength(&self, ranking: u16) -> f64 {
        let capped = ranking.min(self.max_ranking) as f64;
        // Inverse relationship: lower ranking = higher strength
        // Use square root to compress the scale
        1.0 - (capped / self.max_ranking as f64).sqrt()
    }

    /// Calculate strength ratio between two teams.
    fn strength_ratio(&self, home_ranking: u16, away_ranking: u16) -> f64 {
        let home_strength = self.ranking_to_strength(home_ranking);
        let away_strength = self.ranking_to_strength(away_ranking);

        let total = home_strength + away_strength;
        if total == 0.0 {
            0.5
        } else {
            home_strength / total
        }
    }

    /// Calculate draw probability.
    fn draw_probability(&self, ratio: f64) -> f64 {
        let diff = (ratio - 0.5).abs();
        0.28 * (1.0 - diff * 2.0).max(0.0)
    }
}

impl PredictionStrategy for FifaRankingStrategy {
    fn name(&self) -> &str {
        "FIFA Ranking"
    }

    fn predict_probabilities(&self, ctx: &MatchContext) -> MatchProbabilities {
        let ratio = self.strength_ratio(
            ctx.home_team.fifa_ranking,
            ctx.away_team.fifa_ranking,
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
        let ratio = self.strength_ratio(
            ctx.home_team.fifa_ranking,
            ctx.away_team.fifa_ranking,
        );

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

    fn create_team(id: u8, ranking: u16) -> Team {
        Team::new(TeamId(id), format!("Team {}", id), format!("T{:02}", id), Confederation::Uefa)
            .with_fifa_ranking(ranking)
    }

    #[test]
    fn test_top_ranked_favored() {
        let strategy = FifaRankingStrategy::default();
        let ctx = MatchContext::new(
            create_team(0, 1),  // #1 ranked
            create_team(1, 50), // #50 ranked
            false,
        );

        let probs = strategy.predict_probabilities(&ctx);

        assert!(probs.home_win > probs.away_win);
        assert!(probs.home_win > 0.5);
    }

    #[test]
    fn test_equal_rankings() {
        let strategy = FifaRankingStrategy::default();
        let ctx = MatchContext::new(create_team(0, 20), create_team(1, 20), false);

        let probs = strategy.predict_probabilities(&ctx);

        assert!((probs.home_win - probs.away_win).abs() < 0.01);
    }

    #[test]
    fn test_ranking_cap() {
        let strategy = FifaRankingStrategy::default();
        // Both teams ranked beyond max_ranking
        let ctx = MatchContext::new(
            create_team(0, 150),
            create_team(1, 200),
            false,
        );

        let probs = strategy.predict_probabilities(&ctx);

        // Both should be treated as equally weak
        assert!((probs.home_win - probs.away_win).abs() < 0.01);
    }

    #[test]
    fn test_underdog_has_chance() {
        let strategy = FifaRankingStrategy::default();
        let ctx = MatchContext::new(
            create_team(0, 1),  // Top team
            create_team(1, 80), // Much lower ranked
            false,
        );

        let probs = strategy.predict_probabilities(&ctx);

        // Underdog should still have some chance (>5%)
        assert!(probs.away_win > 0.05, "Underdog win prob {} should be > 0.05", probs.away_win);
    }
}
