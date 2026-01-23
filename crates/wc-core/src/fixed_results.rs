//! Fixed match results for predetermined outcomes in simulations.

use std::collections::HashMap;
use std::fmt;

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::group::GroupId;
use crate::knockout::KnockoutRound;
use crate::match_result::{MatchResult, PenaltyResult};
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

    /// Generate a MatchResult from this specification.
    ///
    /// For ExactScore, returns the exact specified result.
    /// For WinnerOnly, generates a plausible random score where the specified team wins.
    ///
    /// # Arguments
    /// * `home_team` - The home/first team
    /// * `away_team` - The away/second team
    /// * `is_knockout` - Whether this is a knockout match (affects penalty handling)
    /// * `rng` - Random number generator for WinnerOnly score generation
    pub fn to_match_result<R: Rng>(
        &self,
        home_team: TeamId,
        away_team: TeamId,
        is_knockout: bool,
        rng: &mut R,
    ) -> MatchResult {
        match self {
            FixedResultSpec::ExactScore {
                home_goals,
                away_goals,
                penalties,
            } => {
                let mut result = MatchResult::new(home_team, away_team, *home_goals, *away_goals);

                // If there are penalties specified, it means the match went to extra time
                if let Some((home_pens, away_pens)) = penalties {
                    result.extra_time = true;
                    result.penalties = Some(PenaltyResult {
                        home_penalties: *home_pens,
                        away_penalties: *away_pens,
                    });
                }

                result
            }
            FixedResultSpec::WinnerOnly { winner } => {
                // Generate a plausible score where the specified team wins
                let winner_is_home = *winner == home_team;

                // Sample base goals from Poisson-like distribution (simplified)
                // Using lambda ≈ 1.2 for realistic soccer scores
                let loser_goals = sample_poisson_simple(rng, 1.0);
                let winner_goals = loser_goals + 1 + sample_poisson_simple(rng, 0.5);

                let (home_goals, away_goals) = if winner_is_home {
                    (winner_goals, loser_goals)
                } else {
                    (loser_goals, winner_goals)
                };

                let mut result = MatchResult::new(home_team, away_team, home_goals, away_goals);

                // For knockout matches, if we somehow generated a draw (shouldn't happen
                // with our logic, but for safety), use penalties
                if is_knockout && home_goals == away_goals {
                    result.extra_time = true;
                    // Generate penalties where winner wins
                    result.penalties = Some(generate_winning_penalties(rng, winner_is_home));
                }

                result
            }
        }
    }
}

/// Sample from a simplified Poisson distribution.
fn sample_poisson_simple<R: Rng>(rng: &mut R, lambda: f64) -> u8 {
    // Knuth algorithm for Poisson sampling
    let l = (-lambda).exp();
    let mut k = 0u8;
    let mut p = 1.0;

    loop {
        p *= rng.gen::<f64>();
        if p <= l {
            break;
        }
        k = k.saturating_add(1);
        if k >= 10 {
            // Cap at reasonable max
            break;
        }
    }

    k
}

/// Generate a penalty shootout result where the specified side wins.
fn generate_winning_penalties<R: Rng>(rng: &mut R, home_wins: bool) -> PenaltyResult {
    // Generate a realistic penalty count
    let winner_pens = 3 + (rng.gen::<f64>() * 3.0) as u8; // 3-5 penalties
    let loser_pens = if winner_pens > 3 {
        winner_pens - 1 - (rng.gen::<f64>() * 2.0) as u8
    } else {
        2 + (rng.gen::<f64>() * 2.0) as u8
    };

    // Ensure winner has more penalties
    let (winner_pens, loser_pens) = if winner_pens > loser_pens {
        (winner_pens, loser_pens)
    } else {
        (loser_pens.saturating_add(1), loser_pens)
    };

    if home_wins {
        PenaltyResult {
            home_penalties: winner_pens,
            away_penalties: loser_pens,
        }
    } else {
        PenaltyResult {
            home_penalties: loser_pens,
            away_penalties: winner_pens,
        }
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

/// Error type for fixed results validation.
#[derive(Debug, Clone)]
pub enum FixedResultsError {
    /// A knockout match is fixed but its prerequisite matches are not.
    MissingDependency {
        /// The fixture that has unmet dependencies.
        fixture: MatchFixture,
        /// The fixtures that need to be fixed first.
        missing: Vec<MatchFixture>,
    },
}

impl fmt::Display for FixedResultsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FixedResultsError::MissingDependency { fixture, missing } => {
                write!(
                    f,
                    "Knockout fixture {:?} requires prerequisite matches to be fixed: {:?}",
                    fixture, missing
                )
            }
        }
    }
}

impl std::error::Error for FixedResultsError {}

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

    /// Validate that knockout match dependencies are satisfied.
    ///
    /// Knockout matches have dependencies on earlier rounds:
    /// - R16 slot N depends on R32 slots 2N and 2N+1
    /// - QF slot N depends on R16 slots 2N and 2N+1
    /// - SF slot N depends on QF slots 2N and 2N+1
    /// - Final slot 0 depends on SF slots 0 and 1
    /// - Third place slot 0 depends on SF slots 0 and 1
    ///
    /// Group stage matches have no dependencies and always pass validation.
    pub fn validate_dependencies(&self) -> Result<(), FixedResultsError> {
        for fixture in self.results.keys() {
            if let MatchFixture::Knockout { round, slot } = fixture {
                let dependencies = get_knockout_dependencies(*round, *slot);
                let missing: Vec<_> = dependencies
                    .into_iter()
                    .filter(|dep| !self.results.contains_key(dep))
                    .collect();

                if !missing.is_empty() {
                    return Err(FixedResultsError::MissingDependency {
                        fixture: *fixture,
                        missing,
                    });
                }
            }
        }
        Ok(())
    }
}

/// Get the prerequisite knockout fixtures for a given knockout match.
fn get_knockout_dependencies(round: KnockoutRound, slot: u8) -> Vec<MatchFixture> {
    match round {
        // R32 has no knockout dependencies (comes from group stage)
        KnockoutRound::RoundOf32 => vec![],

        // R16 slot N depends on R32 slots 2N and 2N+1
        KnockoutRound::RoundOf16 => vec![
            MatchFixture::knockout(KnockoutRound::RoundOf32, slot * 2),
            MatchFixture::knockout(KnockoutRound::RoundOf32, slot * 2 + 1),
        ],

        // QF slot N depends on R16 slots 2N and 2N+1
        KnockoutRound::QuarterFinal => vec![
            MatchFixture::knockout(KnockoutRound::RoundOf16, slot * 2),
            MatchFixture::knockout(KnockoutRound::RoundOf16, slot * 2 + 1),
        ],

        // SF slot N depends on QF slots 2N and 2N+1
        KnockoutRound::SemiFinal => vec![
            MatchFixture::knockout(KnockoutRound::QuarterFinal, slot * 2),
            MatchFixture::knockout(KnockoutRound::QuarterFinal, slot * 2 + 1),
        ],

        // Final depends on both semi-finals
        KnockoutRound::Final => vec![
            MatchFixture::knockout(KnockoutRound::SemiFinal, 0),
            MatchFixture::knockout(KnockoutRound::SemiFinal, 1),
        ],

        // Third place match also depends on both semi-finals
        KnockoutRound::ThirdPlace => vec![
            MatchFixture::knockout(KnockoutRound::SemiFinal, 0),
            MatchFixture::knockout(KnockoutRound::SemiFinal, 1),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

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

    // Task 2.1: Tests for to_match_result

    #[test]
    fn test_exact_score_to_match_result() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let spec = FixedResultSpec::exact_score(3, 1);

        let result = spec.to_match_result(TeamId(0), TeamId(1), false, &mut rng);

        assert_eq!(result.home_team, TeamId(0));
        assert_eq!(result.away_team, TeamId(1));
        assert_eq!(result.home_goals, 3);
        assert_eq!(result.away_goals, 1);
        assert!(!result.extra_time);
        assert!(result.penalties.is_none());
    }

    #[test]
    fn test_exact_score_with_penalties_to_match_result() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let spec = FixedResultSpec::exact_score_with_penalties(2, 2, 5, 4);

        let result = spec.to_match_result(TeamId(0), TeamId(1), true, &mut rng);

        assert_eq!(result.home_goals, 2);
        assert_eq!(result.away_goals, 2);
        assert!(result.extra_time);
        assert!(result.penalties.is_some());

        let pens = result.penalties.unwrap();
        assert_eq!(pens.home_penalties, 5);
        assert_eq!(pens.away_penalties, 4);
        assert_eq!(result.winner(), Some(TeamId(0)));
    }

    #[test]
    fn test_winner_only_home_wins() {
        let spec = FixedResultSpec::winner_only(TeamId(0));

        // Run multiple times to verify winner is always correct
        for seed in 0..20 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let result = spec.to_match_result(TeamId(0), TeamId(1), false, &mut rng);

            assert_eq!(
                result.winner(),
                Some(TeamId(0)),
                "Home team should always win with seed {}",
                seed
            );
            assert!(
                result.home_goals > result.away_goals,
                "Home should have more goals"
            );
        }
    }

    #[test]
    fn test_winner_only_away_wins() {
        let spec = FixedResultSpec::winner_only(TeamId(1));

        // Run multiple times to verify winner is always correct
        for seed in 0..20 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let result = spec.to_match_result(TeamId(0), TeamId(1), false, &mut rng);

            assert_eq!(
                result.winner(),
                Some(TeamId(1)),
                "Away team should always win with seed {}",
                seed
            );
            assert!(
                result.away_goals > result.home_goals,
                "Away should have more goals"
            );
        }
    }

    #[test]
    fn test_winner_only_deterministic() {
        // Same seed should produce same result
        let spec = FixedResultSpec::winner_only(TeamId(0));

        let mut rng1 = ChaCha8Rng::seed_from_u64(42);
        let result1 = spec.to_match_result(TeamId(0), TeamId(1), false, &mut rng1);

        let mut rng2 = ChaCha8Rng::seed_from_u64(42);
        let result2 = spec.to_match_result(TeamId(0), TeamId(1), false, &mut rng2);

        assert_eq!(result1.home_goals, result2.home_goals);
        assert_eq!(result1.away_goals, result2.away_goals);
    }

    // Task 2.2: Tests for dependency validation

    #[test]
    fn test_group_stage_no_dependencies() {
        let mut fixed = FixedResults::new();
        fixed.insert(
            MatchFixture::group_stage(GroupId('A'), TeamId(0), TeamId(1)),
            FixedResultSpec::exact_score(2, 1),
        );

        // Group stage matches should always pass validation
        assert!(fixed.validate_dependencies().is_ok());
    }

    #[test]
    fn test_r32_no_dependencies() {
        let mut fixed = FixedResults::new();
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::RoundOf32, 0),
            FixedResultSpec::winner_only(TeamId(0)),
        );

        // R32 has no knockout dependencies
        assert!(fixed.validate_dependencies().is_ok());
    }

    #[test]
    fn test_r16_missing_dependencies() {
        let mut fixed = FixedResults::new();
        // Fix R16 slot 0 without fixing R32 slots 0 and 1
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::RoundOf16, 0),
            FixedResultSpec::winner_only(TeamId(0)),
        );

        let result = fixed.validate_dependencies();
        assert!(result.is_err());

        if let Err(FixedResultsError::MissingDependency { fixture, missing }) = result {
            assert_eq!(fixture, MatchFixture::knockout(KnockoutRound::RoundOf16, 0));
            assert_eq!(missing.len(), 2);
            assert!(missing.contains(&MatchFixture::knockout(KnockoutRound::RoundOf32, 0)));
            assert!(missing.contains(&MatchFixture::knockout(KnockoutRound::RoundOf32, 1)));
        }
    }

    #[test]
    fn test_r16_with_dependencies() {
        let mut fixed = FixedResults::new();
        // Fix R32 slots 0 and 1 first
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::RoundOf32, 0),
            FixedResultSpec::winner_only(TeamId(0)),
        );
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::RoundOf32, 1),
            FixedResultSpec::winner_only(TeamId(1)),
        );
        // Now fix R16 slot 0
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::RoundOf16, 0),
            FixedResultSpec::winner_only(TeamId(0)),
        );

        assert!(fixed.validate_dependencies().is_ok());
    }

    #[test]
    fn test_final_missing_dependencies() {
        let mut fixed = FixedResults::new();
        // Fix final without semi-finals
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::Final, 0),
            FixedResultSpec::winner_only(TeamId(0)),
        );

        let result = fixed.validate_dependencies();
        assert!(result.is_err());

        if let Err(FixedResultsError::MissingDependency { missing, .. }) = result {
            assert_eq!(missing.len(), 2);
            assert!(missing.contains(&MatchFixture::knockout(KnockoutRound::SemiFinal, 0)));
            assert!(missing.contains(&MatchFixture::knockout(KnockoutRound::SemiFinal, 1)));
        }
    }

    #[test]
    fn test_complete_path_to_final() {
        let mut fixed = FixedResults::new();

        // Fix complete path: R32 → R16 → QF → SF → Final on one side
        // R32 slots 0, 1 → R16 slot 0
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::RoundOf32, 0),
            FixedResultSpec::winner_only(TeamId(0)),
        );
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::RoundOf32, 1),
            FixedResultSpec::winner_only(TeamId(1)),
        );
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::RoundOf16, 0),
            FixedResultSpec::winner_only(TeamId(0)),
        );

        // R32 slots 2, 3 → R16 slot 1
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::RoundOf32, 2),
            FixedResultSpec::winner_only(TeamId(2)),
        );
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::RoundOf32, 3),
            FixedResultSpec::winner_only(TeamId(3)),
        );
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::RoundOf16, 1),
            FixedResultSpec::winner_only(TeamId(2)),
        );

        // QF slot 0 (from R16 slots 0 and 1)
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::QuarterFinal, 0),
            FixedResultSpec::winner_only(TeamId(0)),
        );

        // Same for the other side of the bracket (QF slot 1)
        // R32 slots 4-7 → R16 slots 2-3 → QF slot 1
        for i in 4..8 {
            fixed.insert(
                MatchFixture::knockout(KnockoutRound::RoundOf32, i),
                FixedResultSpec::winner_only(TeamId(i)),
            );
        }
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::RoundOf16, 2),
            FixedResultSpec::winner_only(TeamId(4)),
        );
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::RoundOf16, 3),
            FixedResultSpec::winner_only(TeamId(6)),
        );
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::QuarterFinal, 1),
            FixedResultSpec::winner_only(TeamId(4)),
        );

        // SF slot 0 (from QF slots 0 and 1)
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::SemiFinal, 0),
            FixedResultSpec::winner_only(TeamId(0)),
        );

        // Now we need SF slot 1 for the final
        // QF slots 2 and 3 → SF slot 1
        for i in 8..16 {
            fixed.insert(
                MatchFixture::knockout(KnockoutRound::RoundOf32, i),
                FixedResultSpec::winner_only(TeamId(i)),
            );
        }
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::RoundOf16, 4),
            FixedResultSpec::winner_only(TeamId(8)),
        );
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::RoundOf16, 5),
            FixedResultSpec::winner_only(TeamId(10)),
        );
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::RoundOf16, 6),
            FixedResultSpec::winner_only(TeamId(12)),
        );
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::RoundOf16, 7),
            FixedResultSpec::winner_only(TeamId(14)),
        );
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::QuarterFinal, 2),
            FixedResultSpec::winner_only(TeamId(8)),
        );
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::QuarterFinal, 3),
            FixedResultSpec::winner_only(TeamId(12)),
        );
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::SemiFinal, 1),
            FixedResultSpec::winner_only(TeamId(8)),
        );

        // Now we can fix the final
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::Final, 0),
            FixedResultSpec::winner_only(TeamId(0)),
        );

        assert!(fixed.validate_dependencies().is_ok());
    }
}
