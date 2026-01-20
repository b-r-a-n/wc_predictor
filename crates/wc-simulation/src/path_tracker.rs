//! Tournament path tracking for knockout stages.
//!
//! This module provides data structures for tracking the paths teams take
//! through the knockout stages, including opponent frequencies at each round
//! and complete path statistics.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use wc_core::TeamId;

/// Represents the most frequently occurring complete bracket outcome across all simulations.
/// This ensures each team appears at most once in the bracket (unlike per-slot independent picks).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MostFrequentBracket {
    /// Number of times this exact bracket occurred
    pub count: u32,
    /// Probability of this bracket (count / total_simulations)
    pub probability: f64,
    /// Winners of Round of 32 (16 team IDs, one per match)
    pub round_of_32_winners: Vec<TeamId>,
    /// Winners of Round of 16 (8 team IDs, one per match)
    pub round_of_16_winners: Vec<TeamId>,
    /// Winners of Quarter-finals (4 team IDs, one per match)
    pub quarter_final_winners: Vec<TeamId>,
    /// Winners of Semi-finals (2 team IDs, one per match)
    pub semi_final_winners: Vec<TeamId>,
    /// Tournament champion
    pub champion: TeamId,
}

/// Slot data for most likely bracket display.
/// Contains the team assigned to a slot along with count and probability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MostLikelyBracketSlot {
    /// Team assigned to this slot
    pub team_id: TeamId,
    /// Number of wins (or participations as fallback) at this slot
    pub count: u32,
    /// Probability (count / total_simulations)
    pub probability: f64,
}

/// The most likely bracket computed via greedy algorithm.
/// Ensures each team appears at most once and follows bracket structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MostLikelyBracket {
    /// Round of 32 slot assignments (slot 0-15 -> team data)
    pub round_of_32: HashMap<u8, MostLikelyBracketSlot>,
    /// Round of 16 slot assignments (slot 0-7 -> team data)
    pub round_of_16: HashMap<u8, MostLikelyBracketSlot>,
    /// Quarter-finals slot assignments (slot 0-3 -> team data)
    pub quarter_finals: HashMap<u8, MostLikelyBracketSlot>,
    /// Semi-finals slot assignments (slot 0-1 -> team data)
    pub semi_finals: HashMap<u8, MostLikelyBracketSlot>,
    /// Final winner
    pub final_match: Option<MostLikelyBracketSlot>,
    /// Tournament champion (same as final_match winner)
    pub champion: Option<MostLikelyBracketSlot>,
}

/// Tracks which bracket slots (positions) a team plays in at each knockout round.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BracketSlotStats {
    /// Round of 32 slot appearances (slot 0-15 -> count)
    pub round_of_32: HashMap<u8, u32>,
    /// Round of 16 slot appearances (slot 0-7 -> count)
    pub round_of_16: HashMap<u8, u32>,
    /// Quarter-finals slot appearances (slot 0-3 -> count)
    pub quarter_finals: HashMap<u8, u32>,
    /// Semi-finals slot appearances (slot 0-1 -> count)
    pub semi_finals: HashMap<u8, u32>,
    /// Final appearances count
    pub final_match: u32,
}

impl BracketSlotStats {
    /// Create new empty bracket slot stats.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a slot appearance for a given round.
    pub fn record_slot(&mut self, round: &str, slot: u8) {
        match round {
            "round_of_32" => *self.round_of_32.entry(slot).or_insert(0) += 1,
            "round_of_16" => *self.round_of_16.entry(slot).or_insert(0) += 1,
            "quarter_finals" => *self.quarter_finals.entry(slot).or_insert(0) += 1,
            "semi_finals" => *self.semi_finals.entry(slot).or_insert(0) += 1,
            "final" => self.final_match += 1,
            _ => {}
        }
    }
}

/// Tracks WINS (not just participation) per bracket slot for each team.
/// Only the match winner gets recorded, not the loser.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BracketSlotWinStats {
    /// Round of 32 slot wins (slot 0-15 -> count)
    pub round_of_32: HashMap<u8, u32>,
    /// Round of 16 slot wins (slot 0-7 -> count)
    pub round_of_16: HashMap<u8, u32>,
    /// Quarter-finals slot wins (slot 0-3 -> count)
    pub quarter_finals: HashMap<u8, u32>,
    /// Semi-finals slot wins (slot 0-1 -> count)
    pub semi_finals: HashMap<u8, u32>,
    /// Final wins count (champion)
    pub final_match: u32,
}

impl BracketSlotWinStats {
    /// Create new empty bracket slot win stats.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a win at a specific slot in a round.
    pub fn record_win(&mut self, round: &str, slot: u8) {
        match round {
            "round_of_32" => *self.round_of_32.entry(slot).or_insert(0) += 1,
            "round_of_16" => *self.round_of_16.entry(slot).or_insert(0) += 1,
            "quarter_finals" => *self.quarter_finals.entry(slot).or_insert(0) += 1,
            "semi_finals" => *self.semi_finals.entry(slot).or_insert(0) += 1,
            "final" => self.final_match += 1,
            _ => {}
        }
    }
}

/// Track opponents faced by team in specific bracket slots per round.
/// This enables per-slot opponent statistics (e.g., who did the team face
/// specifically in R32 slot #2, not just "anywhere in R32").
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SlotOpponentStats {
    /// R32: slot (0-15) -> opponent -> count
    pub round_of_32: HashMap<u8, HashMap<TeamId, u32>>,
    /// R16: slot (0-7) -> opponent -> count
    pub round_of_16: HashMap<u8, HashMap<TeamId, u32>>,
    /// QF: slot (0-3) -> opponent -> count
    pub quarter_finals: HashMap<u8, HashMap<TeamId, u32>>,
    /// SF: slot (0-1) -> opponent -> count
    pub semi_finals: HashMap<u8, HashMap<TeamId, u32>>,
    /// Final: opponent -> count (single slot)
    pub final_match: HashMap<TeamId, u32>,
}

impl SlotOpponentStats {
    /// Create new empty slot opponent stats.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an opponent faced at a specific slot in a round.
    pub fn record_opponent(&mut self, round: &str, slot: u8, opponent: TeamId) {
        let slot_map = match round {
            "round_of_32" => &mut self.round_of_32,
            "round_of_16" => &mut self.round_of_16,
            "quarter_finals" => &mut self.quarter_finals,
            "semi_finals" => &mut self.semi_finals,
            _ => return,
        };
        *slot_map.entry(slot).or_default().entry(opponent).or_insert(0) += 1;
    }

    /// Record an opponent faced in the final.
    pub fn record_final_opponent(&mut self, opponent: TeamId) {
        *self.final_match.entry(opponent).or_insert(0) += 1;
    }
}

/// Tracks opponent frequencies at a specific knockout round.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoundMatchups {
    /// Maps opponent TeamId to count of times faced.
    pub opponents: HashMap<TeamId, u32>,
}

impl RoundMatchups {
    /// Record a matchup against an opponent.
    pub fn record_opponent(&mut self, opponent: TeamId) {
        *self.opponents.entry(opponent).or_insert(0) += 1;
    }
}

/// Statistics for tracking tournament paths for a single team.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathStatistics {
    /// Team this data is for.
    pub team_id: TeamId,
    /// Matchup frequencies at Round of 32.
    pub round_of_32_matchups: RoundMatchups,
    /// Matchup frequencies at Round of 16.
    pub round_of_16_matchups: RoundMatchups,
    /// Matchup frequencies at Quarter-finals.
    pub quarter_final_matchups: RoundMatchups,
    /// Matchup frequencies at Semi-finals.
    pub semi_final_matchups: RoundMatchups,
    /// Matchup frequencies at Final.
    pub final_matchups: RoundMatchups,
    /// Top complete paths with frequencies.
    /// Key: Serialized path (e.g., "R32:5,R16:12,QF:3,SF:14,F:0")
    /// Value: Count of times this exact path occurred.
    pub complete_paths: HashMap<String, u32>,
}

impl PathStatistics {
    /// Create new path statistics for a team.
    pub fn new(team_id: TeamId) -> Self {
        Self {
            team_id,
            round_of_32_matchups: RoundMatchups::default(),
            round_of_16_matchups: RoundMatchups::default(),
            quarter_final_matchups: RoundMatchups::default(),
            semi_final_matchups: RoundMatchups::default(),
            final_matchups: RoundMatchups::default(),
            complete_paths: HashMap::new(),
        }
    }

    /// Record matchups at each round for a team's path through the knockout stage.
    /// Returns the path key string for complete path tracking.
    pub fn record_path(
        &mut self,
        r32_opponent: Option<TeamId>,
        r16_opponent: Option<TeamId>,
        qf_opponent: Option<TeamId>,
        sf_opponent: Option<TeamId>,
        final_opponent: Option<TeamId>,
    ) -> String {
        let mut path_parts = Vec::new();

        if let Some(opp) = r32_opponent {
            self.round_of_32_matchups.record_opponent(opp);
            path_parts.push(format!("R32:{}", opp.0));
        }

        if let Some(opp) = r16_opponent {
            self.round_of_16_matchups.record_opponent(opp);
            path_parts.push(format!("R16:{}", opp.0));
        }

        if let Some(opp) = qf_opponent {
            self.quarter_final_matchups.record_opponent(opp);
            path_parts.push(format!("QF:{}", opp.0));
        }

        if let Some(opp) = sf_opponent {
            self.semi_final_matchups.record_opponent(opp);
            path_parts.push(format!("SF:{}", opp.0));
        }

        if let Some(opp) = final_opponent {
            self.final_matchups.record_opponent(opp);
            path_parts.push(format!("F:{}", opp.0));
        }

        path_parts.join(",")
    }

    /// Record a complete path.
    pub fn record_complete_path(&mut self, path_key: String) {
        if !path_key.is_empty() {
            *self.complete_paths.entry(path_key).or_insert(0) += 1;
        }
    }

    /// Prune complete_paths to keep only the top N entries by count.
    pub fn prune_paths(&mut self, max_entries: usize) {
        if self.complete_paths.len() <= max_entries {
            return;
        }

        // Collect entries and sort by count descending
        let mut entries: Vec<_> = self.complete_paths.drain().collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1));

        // Keep only top entries
        self.complete_paths = entries.into_iter().take(max_entries).collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_matchups_record() {
        let mut matchups = RoundMatchups::default();
        matchups.record_opponent(TeamId(5));
        matchups.record_opponent(TeamId(5));
        matchups.record_opponent(TeamId(10));

        assert_eq!(matchups.opponents.get(&TeamId(5)), Some(&2));
        assert_eq!(matchups.opponents.get(&TeamId(10)), Some(&1));
    }

    #[test]
    fn test_path_statistics_record_path() {
        let mut stats = PathStatistics::new(TeamId(0));

        let path = stats.record_path(
            Some(TeamId(1)),
            Some(TeamId(2)),
            Some(TeamId(3)),
            Some(TeamId(4)),
            Some(TeamId(5)),
        );

        assert_eq!(path, "R32:1,R16:2,QF:3,SF:4,F:5");
        assert_eq!(
            stats.round_of_32_matchups.opponents.get(&TeamId(1)),
            Some(&1)
        );
        assert_eq!(
            stats.round_of_16_matchups.opponents.get(&TeamId(2)),
            Some(&1)
        );
        assert_eq!(
            stats.quarter_final_matchups.opponents.get(&TeamId(3)),
            Some(&1)
        );
        assert_eq!(
            stats.semi_final_matchups.opponents.get(&TeamId(4)),
            Some(&1)
        );
        assert_eq!(stats.final_matchups.opponents.get(&TeamId(5)), Some(&1));
    }

    #[test]
    fn test_path_statistics_partial_path() {
        let mut stats = PathStatistics::new(TeamId(0));

        // Team eliminated in R16
        let path = stats.record_path(Some(TeamId(1)), Some(TeamId(2)), None, None, None);

        assert_eq!(path, "R32:1,R16:2");
        assert!(stats.quarter_final_matchups.opponents.is_empty());
    }

    #[test]
    fn test_prune_paths() {
        let mut stats = PathStatistics::new(TeamId(0));

        // Add more than 100 paths
        for i in 0..150 {
            let path_key = format!("path_{}", i);
            // Give earlier paths higher counts
            for _ in 0..(150 - i) {
                stats.record_complete_path(path_key.clone());
            }
        }

        assert_eq!(stats.complete_paths.len(), 150);

        stats.prune_paths(100);

        assert_eq!(stats.complete_paths.len(), 100);

        // Verify the highest count paths are kept
        assert!(stats.complete_paths.contains_key("path_0"));
        assert!(stats.complete_paths.contains_key("path_99"));
        assert!(!stats.complete_paths.contains_key("path_100"));
    }

    #[test]
    fn test_prune_paths_no_op_when_under_limit() {
        let mut stats = PathStatistics::new(TeamId(0));

        for i in 0..50 {
            stats.record_complete_path(format!("path_{}", i));
        }

        stats.prune_paths(100);

        assert_eq!(stats.complete_paths.len(), 50);
    }

    #[test]
    fn test_bracket_slot_stats_new() {
        let stats = BracketSlotStats::new();
        assert!(stats.round_of_32.is_empty());
        assert!(stats.round_of_16.is_empty());
        assert!(stats.quarter_finals.is_empty());
        assert!(stats.semi_finals.is_empty());
        assert_eq!(stats.final_match, 0);
    }

    #[test]
    fn test_bracket_slot_stats_record_slot() {
        let mut stats = BracketSlotStats::new();

        stats.record_slot("round_of_32", 5);
        stats.record_slot("round_of_32", 5);
        stats.record_slot("round_of_32", 10);
        stats.record_slot("round_of_16", 3);
        stats.record_slot("quarter_finals", 1);
        stats.record_slot("semi_finals", 0);
        stats.record_slot("final", 0);
        stats.record_slot("final", 0);

        assert_eq!(stats.round_of_32.get(&5), Some(&2));
        assert_eq!(stats.round_of_32.get(&10), Some(&1));
        assert_eq!(stats.round_of_16.get(&3), Some(&1));
        assert_eq!(stats.quarter_finals.get(&1), Some(&1));
        assert_eq!(stats.semi_finals.get(&0), Some(&1));
        assert_eq!(stats.final_match, 2);
    }

    #[test]
    fn test_bracket_slot_stats_unknown_round() {
        let mut stats = BracketSlotStats::new();
        stats.record_slot("unknown_round", 0);
        // Should not panic and all maps should remain empty
        assert!(stats.round_of_32.is_empty());
        assert!(stats.round_of_16.is_empty());
        assert!(stats.quarter_finals.is_empty());
        assert!(stats.semi_finals.is_empty());
        assert_eq!(stats.final_match, 0);
    }

    #[test]
    fn test_bracket_slot_win_stats_new() {
        let stats = BracketSlotWinStats::new();
        assert!(stats.round_of_32.is_empty());
        assert!(stats.round_of_16.is_empty());
        assert!(stats.quarter_finals.is_empty());
        assert!(stats.semi_finals.is_empty());
        assert_eq!(stats.final_match, 0);
    }

    #[test]
    fn test_bracket_slot_win_stats_record_win() {
        let mut stats = BracketSlotWinStats::new();

        stats.record_win("round_of_32", 5);
        stats.record_win("round_of_32", 5);
        stats.record_win("round_of_32", 10);
        stats.record_win("round_of_16", 3);
        stats.record_win("quarter_finals", 1);
        stats.record_win("semi_finals", 0);
        stats.record_win("final", 0);
        stats.record_win("final", 0);

        assert_eq!(stats.round_of_32.get(&5), Some(&2));
        assert_eq!(stats.round_of_32.get(&10), Some(&1));
        assert_eq!(stats.round_of_16.get(&3), Some(&1));
        assert_eq!(stats.quarter_finals.get(&1), Some(&1));
        assert_eq!(stats.semi_finals.get(&0), Some(&1));
        assert_eq!(stats.final_match, 2);
    }

    #[test]
    fn test_bracket_slot_win_stats_unknown_round() {
        let mut stats = BracketSlotWinStats::new();
        stats.record_win("unknown_round", 0);
        // Should not panic and all maps should remain empty
        assert!(stats.round_of_32.is_empty());
        assert!(stats.round_of_16.is_empty());
        assert!(stats.quarter_finals.is_empty());
        assert!(stats.semi_finals.is_empty());
        assert_eq!(stats.final_match, 0);
    }
}
