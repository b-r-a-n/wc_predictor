//! Tournament path tracking for knockout stages.
//!
//! This module provides data structures for tracking the paths teams take
//! through the knockout stages, including opponent frequencies at each round
//! and complete path statistics.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use wc_core::TeamId;

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
}
