//! Tournament configuration and results.

use serde::{Deserialize, Serialize};

use crate::group::{Group, GroupResult};
use crate::knockout::KnockoutBracket;
use crate::team::{Team, TeamId};

/// World Cup 2026 tournament configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tournament {
    /// All 48 participating teams
    pub teams: Vec<Team>,
    /// The 12 groups (A through L)
    pub groups: Vec<Group>,
}

impl Tournament {
    /// Number of teams in the tournament.
    pub const NUM_TEAMS: usize = 48;
    /// Number of groups.
    pub const NUM_GROUPS: usize = 12;
    /// Teams per group.
    pub const TEAMS_PER_GROUP: usize = 4;
    /// Teams advancing from group stage (top 2 from each + 8 best third).
    pub const ADVANCING_FROM_GROUPS: usize = 32;

    /// Create a new tournament with the given teams and groups.
    pub fn new(teams: Vec<Team>, groups: Vec<Group>) -> Self {
        Self { teams, groups }
    }

    /// Get a team by ID.
    pub fn get_team(&self, id: TeamId) -> Option<&Team> {
        self.teams.iter().find(|t| t.id == id)
    }

    /// Get a team by ID (mutable).
    pub fn get_team_mut(&mut self, id: TeamId) -> Option<&mut Team> {
        self.teams.iter_mut().find(|t| t.id == id)
    }

    /// Get the group containing a specific team.
    pub fn get_team_group(&self, team: TeamId) -> Option<&Group> {
        self.groups.iter().find(|g| g.contains(team))
    }

    /// Validate tournament configuration.
    pub fn validate(&self) -> Result<(), TournamentError> {
        // Check team count
        if self.teams.len() != Self::NUM_TEAMS {
            return Err(TournamentError::InvalidTeamCount(self.teams.len()));
        }

        // Check group count
        if self.groups.len() != Self::NUM_GROUPS {
            return Err(TournamentError::InvalidGroupCount(self.groups.len()));
        }

        // Check each group has 4 teams
        for group in &self.groups {
            // Verify all team IDs in group exist
            for &team_id in &group.teams {
                if self.get_team(team_id).is_none() {
                    return Err(TournamentError::TeamNotFound(team_id));
                }
            }
        }

        // Check no team appears in multiple groups
        let mut seen_teams = std::collections::HashSet::new();
        for group in &self.groups {
            for &team_id in &group.teams {
                if !seen_teams.insert(team_id) {
                    return Err(TournamentError::DuplicateTeam(team_id));
                }
            }
        }

        Ok(())
    }
}

/// Tournament configuration error.
#[derive(Debug, Clone, thiserror::Error)]
pub enum TournamentError {
    #[error("Invalid team count: expected 48, got {0}")]
    InvalidTeamCount(usize),

    #[error("Invalid group count: expected 12, got {0}")]
    InvalidGroupCount(usize),

    #[error("Team not found: {0:?}")]
    TeamNotFound(TeamId),

    #[error("Team appears in multiple groups: {0:?}")]
    DuplicateTeam(TeamId),
}

/// Complete result of a simulated tournament.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentResult {
    /// Results of all 12 groups
    pub group_results: Vec<GroupResult>,
    /// Complete knockout bracket
    pub knockout_bracket: KnockoutBracket,
    /// Tournament champion
    pub champion: TeamId,
    /// Runner-up (finalist)
    pub runner_up: TeamId,
    /// Third place
    pub third_place: TeamId,
    /// Fourth place
    pub fourth_place: TeamId,
}

impl TournamentResult {
    /// Get the top 4 finishers.
    pub fn podium(&self) -> [TeamId; 4] {
        [
            self.champion,
            self.runner_up,
            self.third_place,
            self.fourth_place,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::group::GroupId;
    use crate::team::Confederation;

    fn create_test_teams() -> Vec<Team> {
        (0..48)
            .map(|i| Team::new(TeamId(i), format!("Team {}", i), format!("T{:02}", i), Confederation::Uefa))
            .collect()
    }

    fn create_test_groups(teams: &[Team]) -> Vec<Group> {
        (0..12)
            .map(|i| {
                let start = i * 4;
                Group::new(
                    GroupId::from_index(i as u8),
                    [
                        teams[start].id,
                        teams[start + 1].id,
                        teams[start + 2].id,
                        teams[start + 3].id,
                    ],
                )
            })
            .collect()
    }

    #[test]
    fn test_tournament_validation() {
        let teams = create_test_teams();
        let groups = create_test_groups(&teams);
        let tournament = Tournament::new(teams, groups);

        assert!(tournament.validate().is_ok());
    }

    #[test]
    fn test_tournament_invalid_team_count() {
        let teams: Vec<Team> = (0..40)
            .map(|i| Team::new(TeamId(i), format!("Team {}", i), format!("T{:02}", i), Confederation::Uefa))
            .collect();
        let groups = vec![];
        let tournament = Tournament::new(teams, groups);

        assert!(matches!(
            tournament.validate(),
            Err(TournamentError::InvalidTeamCount(40))
        ));
    }

    #[test]
    fn test_get_team() {
        let teams = create_test_teams();
        let groups = create_test_groups(&teams);
        let tournament = Tournament::new(teams, groups);

        let team = tournament.get_team(TeamId(5)).unwrap();
        assert_eq!(team.name, "Team 5");
    }
}
