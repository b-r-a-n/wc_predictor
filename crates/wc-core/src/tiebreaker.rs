//! FIFA tiebreaker rules for group stage standings.
//!
//! For the 2026 World Cup, FIFA changed the order so that head-to-head results
//! are applied *before* overall goal difference. Teams level on points are
//! ranked by:
//! 1. Points (3 for win, 1 for draw, 0 for loss)
//! 2. Head-to-head points (in matches among the teams level on points)
//! 3. Head-to-head goal difference (among the teams level on points)
//! 4. Head-to-head goals scored (among the teams level on points)
//! 5. Overall goal difference (all group matches)
//! 6. Overall goals scored (all group matches)
//! 7. Fair play points (yellow/red cards) - not simulated
//! 8. Drawing of lots (random) - simulated as tiebreaker

use std::collections::HashMap;

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

/// Head-to-head record for a team, restricted to matches against the other
/// teams it is level on points with.
#[derive(Default, Clone, Copy)]
struct HeadToHead {
    points: i16,
    goal_difference: i16,
    goals_for: i16,
}

/// Compute each team's head-to-head record against the other teams it is level
/// on points with (the FIFA "mini-table" among teams tied on points). Teams not
/// tied with anyone get an all-zero record, which is harmless because the points
/// criterion separates them first.
fn head_to_head_records(
    standings: &[GroupStanding],
    matches: &[MatchResult],
) -> HashMap<TeamId, HeadToHead> {
    let points_by_team: HashMap<TeamId, u8> =
        standings.iter().map(|s| (s.team_id, s.points)).collect();

    let mut records: HashMap<TeamId, HeadToHead> = HashMap::new();
    for s in standings {
        let mut h2h = HeadToHead::default();
        for m in matches {
            let opponent = if m.home_team == s.team_id {
                m.away_team
            } else if m.away_team == s.team_id {
                m.home_team
            } else {
                continue;
            };
            // Only matches against teams level on points count toward H2H.
            if points_by_team.get(&opponent) != Some(&s.points) {
                continue;
            }
            h2h.points += m.points_for(s.team_id) as i16;
            h2h.goal_difference += m.goal_difference(s.team_id);
            h2h.goals_for += m.goals_for(s.team_id) as i16;
        }
        records.insert(s.team_id, h2h);
    }
    records
}

/// Resolve standings with the 2026 FIFA tiebreaker rules.
/// Returns standings sorted from first to last place.
pub fn resolve_standings(
    mut standings: Vec<GroupStanding>,
    matches: &[MatchResult],
) -> Vec<GroupStanding> {
    let h2h = head_to_head_records(&standings, matches);

    standings.sort_by(|a, b| {
        let a_h2h = h2h[&a.team_id];
        let b_h2h = h2h[&b.team_id];
        // 1. Points (descending)
        b.points
            .cmp(&a.points)
            // 2. Head-to-head points among teams level on points (descending)
            .then_with(|| b_h2h.points.cmp(&a_h2h.points))
            // 3. Head-to-head goal difference (descending)
            .then_with(|| b_h2h.goal_difference.cmp(&a_h2h.goal_difference))
            // 4. Head-to-head goals scored (descending)
            .then_with(|| b_h2h.goals_for.cmp(&a_h2h.goals_for))
            // 5. Overall goal difference (descending)
            .then_with(|| b.goal_difference().cmp(&a.goal_difference()))
            // 6. Overall goals scored (descending)
            .then_with(|| b.goals_for.cmp(&a.goals_for))
            // 7. Team ID as final tiebreaker (simulates drawing of lots)
            .then_with(|| a.team_id.0.cmp(&b.team_id.0))
    });

    standings
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

    #[test]
    fn test_head_to_head_beats_overall_goal_difference_2026() {
        // 2026 rule: head-to-head is applied BEFORE overall goal difference.
        // Team 1 has a far better overall goal difference, but Team 0 beat
        // Team 1 head-to-head, so Team 0 must still finish ahead.
        // (This is the Mexico-over-South-Korea case.)
        let matches = vec![MatchResult::new(TeamId(0), TeamId(1), 1, 0)];

        let standings = vec![
            GroupStanding {
                team_id: TeamId(0),
                group_id: GroupId('A'),
                points: 6,
                goals_for: 3,
                goals_against: 2, // GD +1
                played: 3,
                wins: 2,
                draws: 0,
                losses: 1,
            },
            GroupStanding {
                team_id: TeamId(1),
                group_id: GroupId('A'),
                points: 6,
                goals_for: 8,
                goals_against: 2, // GD +6
                played: 3,
                wins: 2,
                draws: 0,
                losses: 1,
            },
        ];

        let resolved = resolve_standings(standings, &matches);
        assert_eq!(resolved[0].team_id, TeamId(0)); // H2H winner first despite worse GD
    }
}
