//! FIFA tiebreaker rules for group stage standings.
//!
//! According to FIFA regulations, teams are ranked by:
//! 1. Points (3 for win, 1 for draw, 0 for loss)
//! 2. Goal difference
//! 3. Goals scored
//! 4. Head-to-head points
//! 5. Head-to-head goal difference
//! 6. Head-to-head goals scored
//! 7. Fair play points (yellow/red cards) - not simulated
//! 8. Drawing of lots (random) - simulated as tiebreaker

use serde::{Deserialize, Serialize};

use crate::group::GroupId;
use crate::match_result::MatchResult;
use crate::team::TeamId;

/// A team's standing within a group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupStanding {
    /// Team identifier
    pub team_id: TeamId,
    /// Group this team belongs to
    pub group_id: GroupId,
    /// Matches played
    pub played: u8,
    /// Matches won
    pub wins: u8,
    /// Matches drawn
    pub draws: u8,
    /// Matches lost
    pub losses: u8,
    /// Total goals scored
    pub goals_for: u16,
    /// Total goals conceded
    pub goals_against: u16,
    /// Total points (3*W + 1*D)
    pub points: u8,
}

impl Default for GroupStanding {
    fn default() -> Self {
        Self {
            team_id: TeamId(0),
            group_id: GroupId('A'),
            played: 0,
            wins: 0,
            draws: 0,
            losses: 0,
            goals_for: 0,
            goals_against: 0,
            points: 0,
        }
    }
}

impl GroupStanding {
    /// Create a new standing for a team in a specific group.
    pub fn new(team_id: TeamId, group_id: GroupId) -> Self {
        Self {
            team_id,
            group_id,
            ..Default::default()
        }
    }

    /// Calculate goal difference.
    pub fn goal_difference(&self) -> i16 {
        self.goals_for as i16 - self.goals_against as i16
    }

    /// Update standing with a match result.
    pub fn add_match(&mut self, result: &MatchResult) {
        self.played += 1;
        self.goals_for += result.goals_for(self.team_id) as u16;
        self.goals_against += result.goals_against(self.team_id) as u16;

        let pts = result.points_for(self.team_id);
        self.points += pts;
        match pts {
            3 => self.wins += 1,
            1 => self.draws += 1,
            0 => self.losses += 1,
            _ => {}
        }
    }
}

/// Calculate standings from match results.
pub fn calculate_standings(teams: &[TeamId], matches: &[MatchResult], group_id: GroupId) -> Vec<GroupStanding> {
    let mut standings: Vec<GroupStanding> = teams.iter().map(|&id| GroupStanding::new(id, group_id)).collect();

    for standing in &mut standings {
        for m in matches {
            if m.home_team == standing.team_id || m.away_team == standing.team_id {
                standing.add_match(m);
            }
        }
    }

    standings
}

/// Resolve standings with FIFA tiebreaker rules.
/// Returns standings sorted from first to last place.
pub fn resolve_standings(
    mut standings: Vec<GroupStanding>,
    matches: &[MatchResult],
) -> Vec<GroupStanding> {
    standings.sort_by(|a, b| {
        // 1. Points (descending)
        b.points
            .cmp(&a.points)
            // 2. Goal difference (descending)
            .then_with(|| b.goal_difference().cmp(&a.goal_difference()))
            // 3. Goals scored (descending)
            .then_with(|| b.goals_for.cmp(&a.goals_for))
            // 4-6. Head-to-head (if still tied)
            .then_with(|| compare_head_to_head(a, b, matches))
            // 7. Team ID as final tiebreaker (simulates drawing of lots)
            .then_with(|| a.team_id.0.cmp(&b.team_id.0))
    });

    standings
}

/// Compare two teams head-to-head based on their direct encounter.
fn compare_head_to_head(
    team_a: &GroupStanding,
    team_b: &GroupStanding,
    matches: &[MatchResult],
) -> std::cmp::Ordering {
    // Find the head-to-head match
    for m in matches {
        let is_match =
            (m.home_team == team_a.team_id && m.away_team == team_b.team_id)
                || (m.home_team == team_b.team_id && m.away_team == team_a.team_id);

        if is_match {
            // Compare points from this match
            let a_points = m.points_for(team_a.team_id);
            let b_points = m.points_for(team_b.team_id);

            if a_points != b_points {
                return b_points.cmp(&a_points);
            }

            // Compare goal difference from this match
            let a_gd = m.goal_difference(team_a.team_id);
            let b_gd = m.goal_difference(team_b.team_id);

            if a_gd != b_gd {
                return b_gd.cmp(&a_gd);
            }

            // Compare goals scored in this match
            let a_goals = m.goals_for(team_a.team_id);
            let b_goals = m.goals_for(team_b.team_id);

            return b_goals.cmp(&a_goals);
        }
    }

    std::cmp::Ordering::Equal
}

/// Rank third-placed teams across all groups to determine the best 8.
/// Returns sorted list of standings (best first).
pub fn rank_third_placed_teams(third_place_standings: &[GroupStanding]) -> Vec<GroupStanding> {
    let mut standings = third_place_standings.to_vec();

    standings.sort_by(|a, b| {
        // 1. Points
        b.points
            .cmp(&a.points)
            // 2. Goal difference
            .then_with(|| b.goal_difference().cmp(&a.goal_difference()))
            // 3. Goals scored
            .then_with(|| b.goals_for.cmp(&a.goals_for))
            // 4. Team ID as tiebreaker
            .then_with(|| a.team_id.0.cmp(&b.team_id.0))
    });

    standings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standing_calculation() {
        let teams = [TeamId(0), TeamId(1)];
        let matches = vec![MatchResult::new(TeamId(0), TeamId(1), 2, 1)];

        let standings = calculate_standings(&teams, &matches, GroupId('A'));

        assert_eq!(standings[0].points, 3);
        assert_eq!(standings[0].wins, 1);
        assert_eq!(standings[0].goals_for, 2);
        assert_eq!(standings[0].goals_against, 1);

        assert_eq!(standings[1].points, 0);
        assert_eq!(standings[1].losses, 1);
    }

    #[test]
    fn test_tiebreaker_points() {
        let teams = [TeamId(0), TeamId(1), TeamId(2), TeamId(3)];
        let matches = vec![
            MatchResult::new(TeamId(0), TeamId(1), 2, 0), // Team 0: 3pts
            MatchResult::new(TeamId(2), TeamId(3), 1, 0), // Team 2: 3pts
            MatchResult::new(TeamId(0), TeamId(2), 1, 1), // Draw
            MatchResult::new(TeamId(1), TeamId(3), 1, 1), // Draw
            MatchResult::new(TeamId(0), TeamId(3), 1, 0), // Team 0: +3pts
            MatchResult::new(TeamId(1), TeamId(2), 0, 2), // Team 2: +3pts
        ];

        let standings = calculate_standings(&teams, &matches, GroupId('A'));
        let resolved = resolve_standings(standings, &matches);

        // Team 0: 7pts (2W, 1D), GD: +3
        // Team 2: 7pts (2W, 1D), GD: +2
        assert_eq!(resolved[0].team_id, TeamId(0));
        assert_eq!(resolved[1].team_id, TeamId(2));
    }

    #[test]
    fn test_head_to_head_tiebreaker() {
        let teams = [TeamId(0), TeamId(1)];
        // Both teams have same points and GD after group, but Team 1 beat Team 0
        let matches = vec![
            MatchResult::new(TeamId(0), TeamId(1), 0, 1), // Team 1 wins H2H
        ];

        let mut standings = vec![
            GroupStanding {
                team_id: TeamId(0),
                group_id: GroupId('A'),
                points: 6,
                goals_for: 5,
                goals_against: 3,
                played: 3,
                wins: 2,
                draws: 0,
                losses: 1,
            },
            GroupStanding {
                team_id: TeamId(1),
                group_id: GroupId('A'),
                points: 6,
                goals_for: 5,
                goals_against: 3,
                played: 3,
                wins: 2,
                draws: 0,
                losses: 1,
            },
        ];

        standings = resolve_standings(standings, &matches);
        assert_eq!(standings[0].team_id, TeamId(1)); // Team 1 won H2H
    }
}
