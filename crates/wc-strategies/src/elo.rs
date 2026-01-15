//! ELO-based prediction strategy.
//!
//! Uses the World Football ELO rating system formula.
//! Reference: https://eloratings.net/about

use crate::traits::{GoalExpectation, MatchContext, MatchProbabilities, PredictionStrategy};

/// ELO-based prediction strategy.
///
/// Calculates win probabilities using the standard ELO formula:
/// We = 1 / (10^(-dr/400) + 1)
///
/// where dr is the rating difference between teams.
#[derive(Debug, Clone)]
pub struct EloStrategy {
    /// Home advantage in ELO points (typically ~100)
    pub home_advantage: f64,
    /// Base goal expectation for average teams
    pub base_goals: f64,
}

impl Default for EloStrategy {
    fn default() -> Self {
        Self {
            home_advantage: 100.0,
            base_goals: 1.3, // Average goals per team in World Cup matches
        }
    }
}

impl EloStrategy {
    /// Create a new ELO strategy with custom parameters.
    pub fn new(home_advantage: f64, base_goals: f64) -> Self {
        Self {
            home_advantage,
            base_goals,
        }
    }

    /// Calculate win expectancy using ELO formula.
    fn win_expectancy(&self, rating_diff: f64) -> f64 {
        1.0 / (1.0 + 10.0_f64.powf(-rating_diff / 400.0))
    }

    /// Calculate draw probability based on rating difference.
    /// Draw probability is highest for evenly matched teams.
    fn draw_probability(&self, rating_diff: f64) -> f64 {
        // Empirically, draw probability peaks around 25-28% for equal teams
        // and decreases as rating difference increases
        let diff_factor = (rating_diff.abs() / 400.0).min(1.0);
        0.28 * (1.0 - diff_factor)
    }
}

impl PredictionStrategy for EloStrategy {
    fn name(&self) -> &str {
        "ELO Rating"
    }

    fn predict_probabilities(&self, ctx: &MatchContext) -> MatchProbabilities {
        let mut rating_diff = ctx.home_team.elo_rating - ctx.away_team.elo_rating;

        // Add home advantage for non-neutral venues
        if !ctx.neutral_venue {
            rating_diff += self.home_advantage;
        }

        let home_we = self.win_expectancy(rating_diff);
        let away_we = 1.0 - home_we;

        // Calculate draw probability
        let draw_prob = if ctx.is_knockout {
            0.0 // No draws in knockout
        } else {
            self.draw_probability(rating_diff)
        };

        // Distribute remaining probability between wins
        let remaining = 1.0 - draw_prob;
        let home_win = home_we * remaining;
        let away_win = away_we * remaining;

        MatchProbabilities::new(home_win, draw_prob, away_win)
    }

    fn predict_goals(&self, ctx: &MatchContext) -> GoalExpectation {
        let probs = self.predict_probabilities(ctx);

        // Convert probabilities to expected goals
        // Teams with higher win probability score more goals on average
        let home_strength = 1.0 + (probs.home_win - 0.33).clamp(-0.3, 0.5);
        let away_strength = 1.0 + (probs.away_win - 0.33).clamp(-0.3, 0.5);

        GoalExpectation::new(
            self.base_goals * home_strength,
            self.base_goals * away_strength,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wc_core::{Confederation, Team, TeamId};

    fn create_team(id: u8, elo: f64) -> Team {
        Team::new(TeamId(id), format!("Team {}", id), format!("T{:02}", id), Confederation::Uefa)
            .with_elo(elo)
    }

    #[test]
    fn test_equal_teams() {
        let strategy = EloStrategy::default();
        let ctx = MatchContext::new(create_team(0, 1800.0), create_team(1, 1800.0), false);

        let probs = strategy.predict_probabilities(&ctx);

        // Equal teams should have ~equal win probabilities
        assert!((probs.home_win - probs.away_win).abs() < 0.1);
        // And significant draw probability
        assert!(probs.draw > 0.2);
    }

    #[test]
    fn test_stronger_team_favored() {
        let strategy = EloStrategy::default();
        let ctx = MatchContext::new(
            create_team(0, 2100.0), // Strong team
            create_team(1, 1600.0), // Weaker team
            false,
        );

        let probs = strategy.predict_probabilities(&ctx);

        assert!(probs.home_win > probs.away_win);
        assert!(probs.home_win > 0.5);
    }

    #[test]
    fn test_knockout_no_draw() {
        let strategy = EloStrategy::default();
        let ctx = MatchContext::new(create_team(0, 1800.0), create_team(1, 1800.0), true);

        let probs = strategy.predict_probabilities(&ctx);

        assert_eq!(probs.draw, 0.0);
        assert!((probs.home_win + probs.away_win - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_goal_expectation() {
        let strategy = EloStrategy::default();
        let ctx = MatchContext::new(create_team(0, 2000.0), create_team(1, 1600.0), false);

        let goals = strategy.predict_goals(&ctx);

        // Stronger team should have higher goal expectation
        assert!(goals.home_lambda > goals.away_lambda);
    }
}
