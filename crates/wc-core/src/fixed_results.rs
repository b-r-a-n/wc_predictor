//! Fixed match results for predetermined outcomes in simulations.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::group::GroupId;
use crate::knockout::KnockoutRound;
use crate::team::TeamId;

/// Identifies a specific match in the tournament.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MatchFixture {
    /// A group stage match identified by group and the two teams.
    GroupStage {
        group_id: GroupId,
        home_team: TeamId,
        away_team: TeamId,
    },
    /// A knockout match identified by round and slot position.
    /// Slot is 0-indexed within the round (0-15 for R32, 0-7 for R16, etc.)
    Knockout {
        round: KnockoutRound,
        slot: u8,
    },
}

impl MatchFixture {
    /// Create a group stage fixture.
    pub fn group_stage(group_id: GroupId, home_team: TeamId, away_team: TeamId) -> Self {
        Self::GroupStage {
            group_id,
            home_team,
            away_team,
        }
    }

    /// Create a knockout fixture.
    pub fn knockout(round: KnockoutRound, slot: u8) -> Self {
        Self::Knockout { round, slot }
    }
}

/// Specification for a fixed match result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode")]
pub enum FixedResultSpec {
    /// Exact score specification.
    ExactScore {
        home_goals: u8,
        away_goals: u8,
        /// Penalty scores if knockout match ended in a draw (home_pens, away_pens).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        penalties: Option<(u8, u8)>,
    },
    /// Only specify the winner; score will be randomly generated.
    WinnerOnly {
        winner: TeamId,
    },
}

impl FixedResultSpec {
    /// Create an exact score specification.
    pub fn exact_score(home_goals: u8, away_goals: u8) -> Self {
        Self::ExactScore {
            home_goals,
            away_goals,
            penalties: None,
        }
    }

    /// Create an exact score specification with penalties.
    pub fn exact_score_with_penalties(
        home_goals: u8,
        away_goals: u8,
        home_pens: u8,
        away_pens: u8,
    ) -> Self {
        Self::ExactScore {
            home_goals,
            away_goals,
            penalties: Some((home_pens, away_pens)),
        }
    }

    /// Create a winner-only specification.
    pub fn winner_only(winner: TeamId) -> Self {
        Self::WinnerOnly { winner }
    }
}

/// A fixed match result combining fixture identification with result specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixedMatchResult {
    pub fixture: MatchFixture,
    pub spec: FixedResultSpec,
}

impl FixedMatchResult {
    /// Create a new fixed match result.
    pub fn new(fixture: MatchFixture, spec: FixedResultSpec) -> Self {
        Self { fixture, spec }
    }
}

/// Collection of fixed match results with lookup and validation methods.
#[derive(Debug, Clone, Default)]
pub struct FixedResults {
    results: HashMap<MatchFixture, FixedResultSpec>,
}

// Custom serialization to use Vec instead of HashMap (JSON requires string keys)
impl Serialize for FixedResults {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.results.len()))?;
        for (fixture, spec) in &self.results {
            seq.serialize_element(&FixedMatchResult {
                fixture: *fixture,
                spec: *spec,
            })?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for FixedResults {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let items: Vec<FixedMatchResult> = Vec::deserialize(deserializer)?;
        let mut results = HashMap::new();
        for item in items {
            results.insert(item.fixture, item.spec);
        }
        Ok(Self { results })
    }
}

impl FixedResults {
    /// Create an empty collection of fixed results.
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
        }
    }

    /// Insert a fixed result for a match.
    pub fn insert(&mut self, fixture: MatchFixture, spec: FixedResultSpec) {
        self.results.insert(fixture, spec);
    }

    /// Get the fixed result for a match, if any.
    pub fn get(&self, fixture: &MatchFixture) -> Option<&FixedResultSpec> {
        self.results.get(fixture)
    }

    /// Get the fixed result for a group stage match.
    pub fn get_group_match(
        &self,
        group_id: GroupId,
        home_team: TeamId,
        away_team: TeamId,
    ) -> Option<&FixedResultSpec> {
        self.results.get(&MatchFixture::GroupStage {
            group_id,
            home_team,
            away_team,
        })
    }

    /// Get the fixed result for a knockout match.
    pub fn get_knockout_match(&self, round: KnockoutRound, slot: u8) -> Option<&FixedResultSpec> {
        self.results.get(&MatchFixture::Knockout { round, slot })
    }

    /// Check if there are no fixed results.
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    /// Get the number of fixed results.
    pub fn len(&self) -> usize {
        self.results.len()
    }

    /// Iterate over all fixed results.
    pub fn iter(&self) -> impl Iterator<Item = (&MatchFixture, &FixedResultSpec)> {
        self.results.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_fixture_group_stage() {
        let fixture = MatchFixture::group_stage(GroupId('A'), TeamId(0), TeamId(1));
        match fixture {
            MatchFixture::GroupStage {
                group_id,
                home_team,
                away_team,
            } => {
                assert_eq!(group_id, GroupId('A'));
                assert_eq!(home_team, TeamId(0));
                assert_eq!(away_team, TeamId(1));
            }
            _ => panic!("Expected GroupStage variant"),
        }
    }

    #[test]
    fn test_match_fixture_knockout() {
        let fixture = MatchFixture::knockout(KnockoutRound::Final, 0);
        match fixture {
            MatchFixture::Knockout { round, slot } => {
                assert_eq!(round, KnockoutRound::Final);
                assert_eq!(slot, 0);
            }
            _ => panic!("Expected Knockout variant"),
        }
    }

    #[test]
    fn test_match_fixture_as_hashmap_key() {
        let mut map: HashMap<MatchFixture, &str> = HashMap::new();
        let fixture1 = MatchFixture::group_stage(GroupId('A'), TeamId(0), TeamId(1));
        let fixture2 = MatchFixture::knockout(KnockoutRound::Final, 0);

        map.insert(fixture1, "group match");
        map.insert(fixture2, "final");

        assert_eq!(map.get(&fixture1), Some(&"group match"));
        assert_eq!(map.get(&fixture2), Some(&"final"));
    }

    #[test]
    fn test_fixed_result_spec_exact_score() {
        let spec = FixedResultSpec::exact_score(2, 1);
        match spec {
            FixedResultSpec::ExactScore {
                home_goals,
                away_goals,
                penalties,
            } => {
                assert_eq!(home_goals, 2);
                assert_eq!(away_goals, 1);
                assert!(penalties.is_none());
            }
            _ => panic!("Expected ExactScore variant"),
        }
    }

    #[test]
    fn test_fixed_result_spec_with_penalties() {
        let spec = FixedResultSpec::exact_score_with_penalties(1, 1, 4, 3);
        match spec {
            FixedResultSpec::ExactScore {
                home_goals,
                away_goals,
                penalties,
            } => {
                assert_eq!(home_goals, 1);
                assert_eq!(away_goals, 1);
                assert_eq!(penalties, Some((4, 3)));
            }
            _ => panic!("Expected ExactScore variant"),
        }
    }

    #[test]
    fn test_fixed_result_spec_winner_only() {
        let spec = FixedResultSpec::winner_only(TeamId(5));
        match spec {
            FixedResultSpec::WinnerOnly { winner } => {
                assert_eq!(winner, TeamId(5));
            }
            _ => panic!("Expected WinnerOnly variant"),
        }
    }

    #[test]
    fn test_fixed_results_collection() {
        let mut fixed = FixedResults::new();
        assert!(fixed.is_empty());
        assert_eq!(fixed.len(), 0);

        let fixture = MatchFixture::group_stage(GroupId('B'), TeamId(4), TeamId(5));
        let spec = FixedResultSpec::exact_score(3, 0);
        fixed.insert(fixture, spec);

        assert!(!fixed.is_empty());
        assert_eq!(fixed.len(), 1);

        // Test direct lookup
        assert!(fixed.get(&fixture).is_some());

        // Test convenience lookup
        let result = fixed.get_group_match(GroupId('B'), TeamId(4), TeamId(5));
        assert!(result.is_some());
        match result.unwrap() {
            FixedResultSpec::ExactScore {
                home_goals,
                away_goals,
                ..
            } => {
                assert_eq!(*home_goals, 3);
                assert_eq!(*away_goals, 0);
            }
            _ => panic!("Expected ExactScore"),
        }

        // Test knockout lookup
        let ko_fixture = MatchFixture::knockout(KnockoutRound::QuarterFinal, 2);
        fixed.insert(ko_fixture, FixedResultSpec::winner_only(TeamId(10)));

        let ko_result = fixed.get_knockout_match(KnockoutRound::QuarterFinal, 2);
        assert!(ko_result.is_some());

        // Test non-existent lookup
        assert!(fixed
            .get_knockout_match(KnockoutRound::Final, 0)
            .is_none());
    }

    #[test]
    fn test_fixed_results_serialization() {
        let mut fixed = FixedResults::new();
        fixed.insert(
            MatchFixture::group_stage(GroupId('A'), TeamId(0), TeamId(1)),
            FixedResultSpec::exact_score(2, 1),
        );
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::Final, 0),
            FixedResultSpec::winner_only(TeamId(0)),
        );

        // Serialize to JSON
        let json = serde_json::to_string(&fixed).expect("Failed to serialize");

        // Deserialize back
        let deserialized: FixedResults =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.len(), 2);
        assert!(deserialized
            .get_group_match(GroupId('A'), TeamId(0), TeamId(1))
            .is_some());
        assert!(deserialized
            .get_knockout_match(KnockoutRound::Final, 0)
            .is_some());
    }
}
