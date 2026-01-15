//! Match result and outcome types.

use serde::{Deserialize, Serialize};

use crate::team::TeamId;

/// Result of a penalty shootout.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PenaltyResult {
    /// Penalties scored by home/first team
    pub home_penalties: u8,
    /// Penalties scored by away/second team
    pub away_penalties: u8,
}

impl PenaltyResult {
    /// Get the winner of the penalty shootout.
    pub fn winner(&self, home_team: TeamId, away_team: TeamId) -> TeamId {
        if self.home_penalties > self.away_penalties {
            home_team
        } else {
            away_team
        }
    }
}

/// Simple match outcome (ignoring score).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchOutcome {
    HomeWin,
    Draw,
    AwayWin,
}

/// Complete result of a match including goals and extra time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    /// Home/first team
    pub home_team: TeamId,
    /// Away/second team
    pub away_team: TeamId,
    /// Goals scored by home team (including extra time)
    pub home_goals: u8,
    /// Goals scored by away team (including extra time)
    pub away_goals: u8,
    /// Whether the match went to extra time
    pub extra_time: bool,
    /// Penalty shootout result (if applicable)
    pub penalties: Option<PenaltyResult>,
}

impl MatchResult {
    /// Create a new match result.
    pub fn new(home_team: TeamId, away_team: TeamId, home_goals: u8, away_goals: u8) -> Self {
        Self {
            home_team,
            away_team,
            home_goals,
            away_goals,
            extra_time: false,
            penalties: None,
        }
    }

    /// Get the match outcome based on goals scored.
    pub fn outcome(&self) -> MatchOutcome {
        match self.home_goals.cmp(&self.away_goals) {
            std::cmp::Ordering::Greater => MatchOutcome::HomeWin,
            std::cmp::Ordering::Less => MatchOutcome::AwayWin,
            std::cmp::Ordering::Equal => MatchOutcome::Draw,
        }
    }

    /// Get the winner of the match (considering penalties for knockout).
    /// Returns None for group stage draws.
    pub fn winner(&self) -> Option<TeamId> {
        match self.outcome() {
            MatchOutcome::HomeWin => Some(self.home_team),
            MatchOutcome::AwayWin => Some(self.away_team),
            MatchOutcome::Draw => self
                .penalties
                .as_ref()
                .map(|p| p.winner(self.home_team, self.away_team)),
        }
    }

    /// Get the loser of the match.
    pub fn loser(&self) -> Option<TeamId> {
        self.winner().map(|w| {
            if w == self.home_team {
                self.away_team
            } else {
                self.home_team
            }
        })
    }

    /// Calculate goal difference for a specific team.
    pub fn goal_difference(&self, team: TeamId) -> i16 {
        if team == self.home_team {
            self.home_goals as i16 - self.away_goals as i16
        } else if team == self.away_team {
            self.away_goals as i16 - self.home_goals as i16
        } else {
            0
        }
    }

    /// Get goals scored by a specific team.
    pub fn goals_for(&self, team: TeamId) -> u8 {
        if team == self.home_team {
            self.home_goals
        } else if team == self.away_team {
            self.away_goals
        } else {
            0
        }
    }

    /// Get goals conceded by a specific team.
    pub fn goals_against(&self, team: TeamId) -> u8 {
        if team == self.home_team {
            self.away_goals
        } else if team == self.away_team {
            self.home_goals
        } else {
            0
        }
    }

    /// Get points earned by a specific team (3 for win, 1 for draw, 0 for loss).
    pub fn points_for(&self, team: TeamId) -> u8 {
        if team == self.home_team {
            match self.outcome() {
                MatchOutcome::HomeWin => 3,
                MatchOutcome::Draw => 1,
                MatchOutcome::AwayWin => 0,
            }
        } else if team == self.away_team {
            match self.outcome() {
                MatchOutcome::AwayWin => 3,
                MatchOutcome::Draw => 1,
                MatchOutcome::HomeWin => 0,
            }
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_outcome() {
        let result = MatchResult::new(TeamId(0), TeamId(1), 2, 1);
        assert_eq!(result.outcome(), MatchOutcome::HomeWin);
        assert_eq!(result.winner(), Some(TeamId(0)));
        assert_eq!(result.loser(), Some(TeamId(1)));
    }

    #[test]
    fn test_goal_difference() {
        let result = MatchResult::new(TeamId(0), TeamId(1), 3, 1);
        assert_eq!(result.goal_difference(TeamId(0)), 2);
        assert_eq!(result.goal_difference(TeamId(1)), -2);
    }

    #[test]
    fn test_points() {
        let win = MatchResult::new(TeamId(0), TeamId(1), 2, 0);
        assert_eq!(win.points_for(TeamId(0)), 3);
        assert_eq!(win.points_for(TeamId(1)), 0);

        let draw = MatchResult::new(TeamId(0), TeamId(1), 1, 1);
        assert_eq!(draw.points_for(TeamId(0)), 1);
        assert_eq!(draw.points_for(TeamId(1)), 1);
    }

    #[test]
    fn test_penalty_winner() {
        let mut result = MatchResult::new(TeamId(0), TeamId(1), 1, 1);
        result.extra_time = true;
        result.penalties = Some(PenaltyResult {
            home_penalties: 4,
            away_penalties: 3,
        });
        assert_eq!(result.winner(), Some(TeamId(0)));
    }
}
