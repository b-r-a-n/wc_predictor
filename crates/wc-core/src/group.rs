//! Group stage types and logic.

use serde::{Deserialize, Serialize};

use crate::match_result::MatchResult;
use crate::team::TeamId;
use crate::tiebreaker::GroupStanding;

/// Group identifier (A through L for 12 groups).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GroupId(pub char);

impl GroupId {
    /// Create a new group ID from index (0-11).
    pub fn from_index(index: u8) -> Self {
        Self((b'A' + index) as char)
    }

    /// Get the index (0-11) from the group ID.
    pub fn to_index(self) -> u8 {
        (self.0 as u8) - b'A'
    }
}

/// A group of 4 teams.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    /// Group identifier (A-L)
    pub id: GroupId,
    /// The 4 teams in this group
    pub teams: [TeamId; 4],
}

impl Group {
    /// Create a new group.
    pub fn new(id: GroupId, teams: [TeamId; 4]) -> Self {
        Self { id, teams }
    }

    /// Generate all 6 round-robin fixtures for this group.
    /// Returns pairs of (home_team, away_team).
    pub fn generate_fixtures(&self) -> Vec<(TeamId, TeamId)> {
        vec![
            // Matchday 1
            (self.teams[0], self.teams[1]),
            (self.teams[2], self.teams[3]),
            // Matchday 2
            (self.teams[0], self.teams[2]),
            (self.teams[1], self.teams[3]),
            // Matchday 3
            (self.teams[0], self.teams[3]),
            (self.teams[1], self.teams[2]),
        ]
    }

    /// Check if a team is in this group.
    pub fn contains(&self, team: TeamId) -> bool {
        self.teams.contains(&team)
    }
}

/// Results of a completed group stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupResult {
    /// Group identifier
    pub group_id: GroupId,
    /// All 6 matches played in this group
    pub matches: Vec<MatchResult>,
    /// Final standings (sorted by position: winner first)
    pub standings: Vec<GroupStanding>,
}

impl GroupResult {
    /// Get the group winner.
    pub fn winner(&self) -> TeamId {
        self.standings[0].team_id
    }

    /// Get the runner-up.
    pub fn runner_up(&self) -> TeamId {
        self.standings[1].team_id
    }

    /// Get the third-placed team.
    pub fn third_place(&self) -> TeamId {
        self.standings[2].team_id
    }

    /// Get the fourth-placed team (eliminated).
    pub fn fourth_place(&self) -> TeamId {
        self.standings[3].team_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_fixtures() {
        let group = Group::new(
            GroupId('A'),
            [TeamId(0), TeamId(1), TeamId(2), TeamId(3)],
        );
        let fixtures = group.generate_fixtures();
        assert_eq!(fixtures.len(), 6);

        // Each team should play 3 matches
        for team_id in 0..4 {
            let matches = fixtures
                .iter()
                .filter(|(h, a)| h.0 == team_id || a.0 == team_id)
                .count();
            assert_eq!(matches, 3);
        }
    }

    #[test]
    fn test_group_id_conversion() {
        assert_eq!(GroupId::from_index(0), GroupId('A'));
        assert_eq!(GroupId::from_index(11), GroupId('L'));
        assert_eq!(GroupId('A').to_index(), 0);
        assert_eq!(GroupId('L').to_index(), 11);
    }
}
