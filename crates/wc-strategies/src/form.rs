//! Sofascore form-based prediction strategy.
//!
//! Uses recent match form (points per game) to predict outcomes.

use crate::traits::{GoalExpectation, MatchContext, MatchProbabilities, PredictionStrategy};

/// Form-based prediction strategy using Sofascore form ratings.
///
/// Form is measured as average points per game from recent matches (0-3 scale).
/// Higher form indicates better recent performance.
#[derive(Debug, Clone)]
pub struct FormStrategy {
    /// Base goal expectation for average teams
    pub base_goals: f64,
}

impl Default for FormStrategy {
    fn default() -> Self {
        Self { base_goals: 1.3 }
    }
}

impl FormStrategy {
    /// Create a new strategy with custom base goals.
    pub fn new(base_goals: f64) -> Self {
        Self { base_goals }
    }

    /// Calculate strength ratio between two teams based on form.
    ///
    /// Form values range from 0.0 (all losses) to 3.0 (all wins).
    fn strength_ratio(&self, home_form: f64, away_form: f64) -> f64 {
        // Clamp form values to valid range
        let home = home_form.clamp(0.0, 3.0);
        let away = away_form.clamp(0.0, 3.0);

        // Add small base to avoid division issues with 0 form
        let home_strength = home + 0.5;
        let away_strength = away + 0.5;

        let total = home_strength + away_strength;
        home_strength / total
    }

    /// Calculate draw probability based on form difference.
    fn draw_probability(&self, ratio: f64) -> f64 {
        let diff = (ratio - 0.5).abs();
        // Higher draw chance when teams have similar form
        0.28 * (1.0 - diff * 2.0).max(0.0)
    }
}

impl PredictionStrategy for FormStrategy {
    fn name(&self) -> &str {
        "Sofascore Form"
    }

    fn predict_probabilities(&self, ctx: &MatchContext) -> MatchProbabilities {
        let ratio = self.strength_ratio(
            ctx.home_team.sofascore_form,
            ctx.away_team.sofascore_form,
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
            ctx.home_team.sofascore_form,
            ctx.away_team.sofascore_form,
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

    fn create_team(id: u8, form: f64) -> Team {
        Team::new(
            TeamId(id),
            format!("Team {}", id),
            format!("T{:02}", id),
            Confederation::Uefa,
        )
        .with_sofascore_form(form)
    }

    #[test]
    fn test_better_form_favored() {
        let strategy = FormStrategy::default();
        let ctx = MatchContext::new(
            create_team(0, 3.0), // Perfect form (all wins)
            create_team(1, 1.0), // Poor form
            false,
        );

        let probs = strategy.predict_probabilities(&ctx);

        assert!(probs.home_win > probs.away_win);
        assert!(probs.home_win > 0.5);
    }

    #[test]
    fn test_equal_form() {
        let strategy = FormStrategy::default();
        let ctx = MatchContext::new(create_team(0, 2.0), create_team(1, 2.0), false);

        let probs = strategy.predict_probabilities(&ctx);

        assert!((probs.home_win - probs.away_win).abs() < 0.01);
    }

    #[test]
    fn test_poor_form_still_has_chance() {
        let strategy = FormStrategy::default();
        let ctx = MatchContext::new(
            create_team(0, 3.0), // Perfect form
            create_team(1, 0.5), // Very poor form
            false,
        );

        let probs = strategy.predict_probabilities(&ctx);

        // Underdog should still have some chance (>10%)
        assert!(
            probs.away_win > 0.10,
            "Underdog win prob {} should be > 0.10",
            probs.away_win
        );
    }

    #[test]
    fn test_knockout_no_draw() {
        let strategy = FormStrategy::default();
        let ctx = MatchContext::new(create_team(0, 2.0), create_team(1, 2.0), true);

        let probs = strategy.predict_probabilities(&ctx);

        assert_eq!(probs.draw, 0.0);
        assert!((probs.home_win + probs.away_win - 1.0).abs() < 0.001);
    }
}
