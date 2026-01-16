//! Aggregation of simulation results.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use wc_core::{MatchResult, TeamId, Tournament, TournamentResult};

use crate::path_tracker::{BracketSlotStats, PathStatistics};

/// Aggregated statistics from multiple tournament simulations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedResults {
    /// Total number of simulations run
    pub total_simulations: u32,
    /// Statistics for each team
    pub team_stats: HashMap<TeamId, TeamStatistics>,
    /// Most likely tournament winner
    pub most_likely_winner: TeamId,
    /// Most likely final matchup
    pub most_likely_final: (TeamId, TeamId),
    /// Path statistics for each team through knockout stages
    pub path_stats: HashMap<TeamId, PathStatistics>,
    /// Bracket slot statistics for each team (which positions they play in)
    pub bracket_slot_stats: HashMap<TeamId, BracketSlotStats>,
}

/// Statistics for a single team across all simulations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamStatistics {
    /// Team ID
    pub team_id: TeamId,
    /// Team name
    pub team_name: String,

    // Group stage outcomes
    /// Times finished as group winner
    pub group_wins: u32,
    /// Times finished second in group
    pub group_second: u32,
    /// Times finished third and qualified
    pub group_third_qualified: u32,
    /// Times eliminated in group stage
    pub group_eliminated: u32,

    // Knockout rounds reached
    /// Times reached Round of 32
    pub reached_round_of_32: u32,
    /// Times reached Round of 16
    pub reached_round_of_16: u32,
    /// Times reached Quarter-finals
    pub reached_quarter_finals: u32,
    /// Times reached Semi-finals
    pub reached_semi_finals: u32,
    /// Times reached Final
    pub reached_final: u32,

    // Final positions
    /// Times won the tournament
    pub champion: u32,
    /// Times finished as runner-up
    pub runner_up: u32,
    /// Times finished third
    pub third_place: u32,
    /// Times finished fourth
    pub fourth_place: u32,

    // Calculated probabilities
    /// Probability of winning the tournament
    pub win_probability: f64,
    /// Probability of reaching the final
    pub final_probability: f64,
    /// Probability of reaching the semi-finals
    pub semi_final_probability: f64,
    /// Probability of advancing from group stage
    pub knockout_probability: f64,
}

impl AggregatedResults {
    /// Aggregate results from multiple tournament simulations.
    pub fn from_results(results: Vec<TournamentResult>, tournament: &Tournament) -> Self {
        let total = results.len() as u32;
        let mut team_stats: HashMap<TeamId, TeamStatistics> = HashMap::new();
        let mut finals_count: HashMap<(TeamId, TeamId), u32> = HashMap::new();
        let mut path_stats: HashMap<TeamId, PathStatistics> = HashMap::new();
        let mut bracket_slot_stats: HashMap<TeamId, BracketSlotStats> = HashMap::new();

        // Initialize stats for all teams
        for team in &tournament.teams {
            team_stats.insert(
                team.id,
                TeamStatistics {
                    team_id: team.id,
                    team_name: team.name.clone(),
                    ..Default::default()
                },
            );
            path_stats.insert(team.id, PathStatistics::new(team.id));
            bracket_slot_stats.insert(team.id, BracketSlotStats::new());
        }

        // Aggregate results
        for result in &results {
            // Track final matchup
            let final_key = if result.champion < result.runner_up {
                (result.champion, result.runner_up)
            } else {
                (result.runner_up, result.champion)
            };
            *finals_count.entry(final_key).or_insert(0) += 1;

            // Track champion (reached all rounds)
            if let Some(stats) = team_stats.get_mut(&result.champion) {
                stats.champion += 1;
                stats.reached_final += 1;
                stats.reached_semi_finals += 1;
                stats.reached_quarter_finals += 1;
                stats.reached_round_of_16 += 1;
                stats.reached_round_of_32 += 1;
            }

            // Track runner-up
            if let Some(stats) = team_stats.get_mut(&result.runner_up) {
                stats.runner_up += 1;
                stats.reached_final += 1;
                stats.reached_semi_finals += 1;
                stats.reached_quarter_finals += 1;
                stats.reached_round_of_16 += 1;
                stats.reached_round_of_32 += 1;
            }

            // Track third place
            if let Some(stats) = team_stats.get_mut(&result.third_place) {
                stats.third_place += 1;
                stats.reached_semi_finals += 1;
                stats.reached_quarter_finals += 1;
                stats.reached_round_of_16 += 1;
                stats.reached_round_of_32 += 1;
            }

            // Track fourth place
            if let Some(stats) = team_stats.get_mut(&result.fourth_place) {
                stats.fourth_place += 1;
                stats.reached_semi_finals += 1;
                stats.reached_quarter_finals += 1;
                stats.reached_round_of_16 += 1;
                stats.reached_round_of_32 += 1;
            }

            // Track group stage results
            let mut knockout_qualifiers: Vec<TeamId> = Vec::with_capacity(32);
            for gr in &result.group_results {
                // Group winner
                if let Some(stats) = team_stats.get_mut(&gr.standings[0].team_id) {
                    stats.group_wins += 1;
                }
                knockout_qualifiers.push(gr.standings[0].team_id);

                // Runner-up
                if let Some(stats) = team_stats.get_mut(&gr.standings[1].team_id) {
                    stats.group_second += 1;
                }
                knockout_qualifiers.push(gr.standings[1].team_id);

                // Fourth place (eliminated)
                if let Some(stats) = team_stats.get_mut(&gr.standings[3].team_id) {
                    stats.group_eliminated += 1;
                }
            }

            // Collect and rank third-placed teams
            let third_placed: Vec<_> = result
                .group_results
                .iter()
                .map(|gr| gr.standings[2].clone())
                .collect();
            let ranked_third = wc_core::tiebreaker::rank_third_placed_teams(&third_placed);

            // Best 8 third-placed teams qualify
            for (i, standing) in ranked_third.iter().enumerate() {
                if let Some(stats) = team_stats.get_mut(&standing.team_id) {
                    if i < 8 {
                        stats.group_third_qualified += 1;
                        knockout_qualifiers.push(standing.team_id);
                    } else {
                        stats.group_eliminated += 1;
                    }
                }
            }

            // Track knockout round participation for teams not in top 4
            // (Top 4 are already tracked above)
            let top_4 = [
                result.champion,
                result.runner_up,
                result.third_place,
                result.fourth_place,
            ];

            // Track Round of 32 participation
            for team_id in &knockout_qualifiers {
                if !top_4.contains(team_id) {
                    if let Some(stats) = team_stats.get_mut(team_id) {
                        stats.reached_round_of_32 += 1;
                    }
                }
            }

            // Track later rounds from knockout bracket results
            for m in &result.knockout_bracket.round_of_32 {
                if let Some(winner) = m.winner() {
                    if !top_4.contains(&winner) {
                        if let Some(stats) = team_stats.get_mut(&winner) {
                            stats.reached_round_of_16 += 1;
                        }
                    }
                }
            }

            for m in &result.knockout_bracket.round_of_16 {
                if let Some(winner) = m.winner() {
                    if !top_4.contains(&winner) {
                        if let Some(stats) = team_stats.get_mut(&winner) {
                            stats.reached_quarter_finals += 1;
                        }
                    }
                }
            }

            for m in &result.knockout_bracket.quarter_finals {
                if let Some(winner) = m.winner() {
                    if !top_4.contains(&winner) {
                        if let Some(stats) = team_stats.get_mut(&winner) {
                            stats.reached_semi_finals += 1;
                        }
                    }
                }
            }

            // Track bracket slot positions for each round
            // Round of 32: slots 0-15
            for (slot, m) in result.knockout_bracket.round_of_32.iter().enumerate() {
                if let Some(stats) = bracket_slot_stats.get_mut(&m.home_team) {
                    stats.record_slot("round_of_32", slot as u8);
                }
                if let Some(stats) = bracket_slot_stats.get_mut(&m.away_team) {
                    stats.record_slot("round_of_32", slot as u8);
                }
            }

            // Round of 16: slots 0-7
            for (slot, m) in result.knockout_bracket.round_of_16.iter().enumerate() {
                if let Some(stats) = bracket_slot_stats.get_mut(&m.home_team) {
                    stats.record_slot("round_of_16", slot as u8);
                }
                if let Some(stats) = bracket_slot_stats.get_mut(&m.away_team) {
                    stats.record_slot("round_of_16", slot as u8);
                }
            }

            // Quarter-finals: slots 0-3
            for (slot, m) in result.knockout_bracket.quarter_finals.iter().enumerate() {
                if let Some(stats) = bracket_slot_stats.get_mut(&m.home_team) {
                    stats.record_slot("quarter_finals", slot as u8);
                }
                if let Some(stats) = bracket_slot_stats.get_mut(&m.away_team) {
                    stats.record_slot("quarter_finals", slot as u8);
                }
            }

            // Semi-finals: slots 0-1
            for (slot, m) in result.knockout_bracket.semi_finals.iter().enumerate() {
                if let Some(stats) = bracket_slot_stats.get_mut(&m.home_team) {
                    stats.record_slot("semi_finals", slot as u8);
                }
                if let Some(stats) = bracket_slot_stats.get_mut(&m.away_team) {
                    stats.record_slot("semi_finals", slot as u8);
                }
            }

            // Final: single match
            {
                let m = &result.knockout_bracket.final_match;
                if let Some(stats) = bracket_slot_stats.get_mut(&m.home_team) {
                    stats.record_slot("final", 0);
                }
                if let Some(stats) = bracket_slot_stats.get_mut(&m.away_team) {
                    stats.record_slot("final", 0);
                }
            }

            // Track path statistics for all knockout qualifiers
            for &team_id in &knockout_qualifiers {
                // Find opponent at each round (team participated if they're in a match)
                let r32_opponent = find_opponent(&result.knockout_bracket.round_of_32, team_id);
                let r16_opponent = find_opponent(&result.knockout_bracket.round_of_16, team_id);
                let qf_opponent = find_opponent(&result.knockout_bracket.quarter_finals, team_id);
                let sf_opponent = find_opponent(&result.knockout_bracket.semi_finals, team_id);
                let final_opponent =
                    find_opponent(std::slice::from_ref(&result.knockout_bracket.final_match), team_id);

                if let Some(stats) = path_stats.get_mut(&team_id) {
                    let path_key = stats.record_path(
                        r32_opponent,
                        r16_opponent,
                        qf_opponent,
                        sf_opponent,
                        final_opponent,
                    );
                    stats.record_complete_path(path_key);
                }
            }
        }

        // Calculate probabilities
        for stats in team_stats.values_mut() {
            stats.win_probability = stats.champion as f64 / total as f64;
            stats.final_probability = stats.reached_final as f64 / total as f64;
            stats.semi_final_probability = stats.reached_semi_finals as f64 / total as f64;
            stats.knockout_probability = stats.reached_round_of_32 as f64 / total as f64;
        }

        // Prune path statistics to top 100 entries per team
        for stats in path_stats.values_mut() {
            stats.prune_paths(100);
        }

        // Find most likely winner
        let most_likely_winner = team_stats
            .values()
            .max_by_key(|s| s.champion)
            .map(|s| s.team_id)
            .unwrap_or(TeamId(0));

        // Find most likely final
        let most_likely_final = finals_count
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(teams, _)| teams)
            .unwrap_or((TeamId(0), TeamId(1)));

        Self {
            total_simulations: total,
            team_stats,
            most_likely_winner,
            most_likely_final,
            path_stats,
            bracket_slot_stats,
        }
    }

    /// Get teams sorted by win probability (highest first).
    pub fn rankings(&self) -> Vec<&TeamStatistics> {
        let mut stats: Vec<_> = self.team_stats.values().collect();
        stats.sort_by(|a, b| {
            b.win_probability
                .partial_cmp(&a.win_probability)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        stats
    }

    /// Get the top N teams by win probability.
    pub fn top_n(&self, n: usize) -> Vec<&TeamStatistics> {
        self.rankings().into_iter().take(n).collect()
    }
}

/// Find the opponent for a team in a list of matches.
/// Returns Some(opponent_id) if the team participated in a match in this round.
fn find_opponent(matches: &[MatchResult], team_id: TeamId) -> Option<TeamId> {
    for m in matches {
        if m.home_team == team_id {
            return Some(m.away_team);
        }
        if m.away_team == team_id {
            return Some(m.home_team);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use wc_core::{
        Confederation, Group, GroupId, GroupResult, GroupStanding, KnockoutBracket, MatchResult,
        Team,
    };

    fn create_test_tournament() -> Tournament {
        let teams: Vec<Team> = (0..48)
            .map(|i| {
                Team::new(
                    TeamId(i),
                    format!("Team {}", i),
                    format!("T{:02}", i),
                    Confederation::Uefa,
                )
            })
            .collect();

        let groups: Vec<Group> = (0..12)
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
            .collect();

        Tournament::new(teams, groups)
    }

    fn create_dummy_tournament_result() -> TournamentResult {
        // Create minimal valid result for testing
        let group_results: Vec<GroupResult> = (0..12)
            .map(|i| {
                let start = (i * 4) as u8;
                GroupResult {
                    group_id: GroupId::from_index(i as u8),
                    matches: vec![],
                    standings: vec![
                        GroupStanding::new(TeamId(start)),
                        GroupStanding::new(TeamId(start + 1)),
                        GroupStanding::new(TeamId(start + 2)),
                        GroupStanding::new(TeamId(start + 3)),
                    ],
                }
            })
            .collect();

        let knockout_bracket = KnockoutBracket {
            round_of_32: (0..16)
                .map(|i| MatchResult::new(TeamId(i * 2), TeamId(i * 2 + 1), 1, 0))
                .collect(),
            round_of_16: (0..8)
                .map(|i| MatchResult::new(TeamId(i * 4), TeamId(i * 4 + 2), 1, 0))
                .collect(),
            quarter_finals: (0..4)
                .map(|i| MatchResult::new(TeamId(i * 8), TeamId(i * 8 + 4), 1, 0))
                .collect(),
            semi_finals: vec![
                MatchResult::new(TeamId(0), TeamId(8), 2, 1),
                MatchResult::new(TeamId(16), TeamId(24), 1, 0),
            ],
            third_place: MatchResult::new(TeamId(8), TeamId(24), 2, 0),
            final_match: MatchResult::new(TeamId(0), TeamId(16), 3, 1),
        };

        TournamentResult {
            group_results,
            knockout_bracket,
            champion: TeamId(0),
            runner_up: TeamId(16),
            third_place: TeamId(8),
            fourth_place: TeamId(24),
        }
    }

    #[test]
    fn test_aggregation_basic() {
        let tournament = create_test_tournament();
        let results = vec![create_dummy_tournament_result()];

        let aggregated = AggregatedResults::from_results(results, &tournament);

        assert_eq!(aggregated.total_simulations, 1);
        assert_eq!(aggregated.team_stats.len(), 48);
        assert_eq!(aggregated.most_likely_winner, TeamId(0));
    }

    #[test]
    fn test_rankings() {
        let tournament = create_test_tournament();
        let results = vec![create_dummy_tournament_result()];

        let aggregated = AggregatedResults::from_results(results, &tournament);
        let rankings = aggregated.rankings();

        // Champion should be first
        assert_eq!(rankings[0].team_id, TeamId(0));
        assert_eq!(rankings[0].win_probability, 1.0);
    }

    #[test]
    fn test_path_statistics_tracking() {
        let tournament = create_test_tournament();
        let results = vec![create_dummy_tournament_result()];

        let aggregated = AggregatedResults::from_results(results, &tournament);

        // path_stats should be populated for all teams
        assert_eq!(aggregated.path_stats.len(), 48);

        // Check champion (Team 0) path statistics
        // Based on the knockout bracket setup:
        // R32: Team 0 vs Team 1 (0 wins)
        // R16: Team 0 vs Team 2 (0 wins)
        // QF: Team 0 vs Team 4 (0 wins)
        // SF: Team 0 vs Team 8 (0 wins)
        // F: Team 0 vs Team 16 (0 wins)
        let team_0_path = aggregated.path_stats.get(&TeamId(0)).unwrap();
        assert_eq!(
            team_0_path.round_of_32_matchups.opponents.get(&TeamId(1)),
            Some(&1)
        );
        assert_eq!(
            team_0_path.round_of_16_matchups.opponents.get(&TeamId(2)),
            Some(&1)
        );
        assert_eq!(
            team_0_path.quarter_final_matchups.opponents.get(&TeamId(4)),
            Some(&1)
        );
        assert_eq!(
            team_0_path.semi_final_matchups.opponents.get(&TeamId(8)),
            Some(&1)
        );
        assert_eq!(
            team_0_path.final_matchups.opponents.get(&TeamId(16)),
            Some(&1)
        );

        // Team 0 should have one complete path
        assert_eq!(team_0_path.complete_paths.len(), 1);
        let expected_path = "R32:1,R16:2,QF:4,SF:8,F:16";
        assert_eq!(
            team_0_path.complete_paths.get(expected_path),
            Some(&1)
        );

        // Check a team eliminated in R32 (Team 1)
        // Team 1 played Team 0 in R32 and lost
        let team_1_path = aggregated.path_stats.get(&TeamId(1)).unwrap();
        assert_eq!(
            team_1_path.round_of_32_matchups.opponents.get(&TeamId(0)),
            Some(&1)
        );
        // Team 1 should not have any R16+ matchups
        assert!(team_1_path.round_of_16_matchups.opponents.is_empty());
        // Team 1's complete path should just be R32
        let expected_team_1_path = "R32:0";
        assert_eq!(
            team_1_path.complete_paths.get(expected_team_1_path),
            Some(&1)
        );
    }

    #[test]
    fn test_path_stats_serialization() {
        use crate::runner::{SimulationConfig, SimulationRunner};
        use wc_strategies::EloStrategy;

        let tournament = create_test_tournament();
        let strategy = EloStrategy::default();
        let config = SimulationConfig::with_iterations(10).with_seed(42);
        let runner = SimulationRunner::new(&tournament, &strategy, config);
        let results = runner.run_with_progress(|_, _| {});

        let json = serde_json::to_string(&results).unwrap();
        // Find path_stats position
        let path_stats_pos = json.find("path_stats").expect("path_stats not found!");
        println!("path_stats at position: {}", path_stats_pos);
        println!("Around path_stats: {}", &json[path_stats_pos..json.len().min(path_stats_pos + 200)]);
        assert!(json.contains("path_stats"), "path_stats should be in JSON output");
    }

    #[test]
    fn test_bracket_slot_stats_tracking() {
        let tournament = create_test_tournament();
        let results = vec![create_dummy_tournament_result()];

        let aggregated = AggregatedResults::from_results(results, &tournament);

        // bracket_slot_stats should be populated for all teams
        assert_eq!(aggregated.bracket_slot_stats.len(), 48);

        // Check Team 0's bracket slot stats
        // Based on the knockout bracket setup:
        // R32: match 0 (Team 0 vs Team 1) -> slot 0
        // R16: match 0 (Team 0 vs Team 2) -> slot 0
        // QF: match 0 (Team 0 vs Team 4) -> slot 0
        // SF: match 0 (Team 0 vs Team 8) -> slot 0
        // Final: Team 0 vs Team 16
        let team_0_slots = aggregated.bracket_slot_stats.get(&TeamId(0)).unwrap();
        assert_eq!(team_0_slots.round_of_32.get(&0), Some(&1));
        assert_eq!(team_0_slots.round_of_16.get(&0), Some(&1));
        assert_eq!(team_0_slots.quarter_finals.get(&0), Some(&1));
        assert_eq!(team_0_slots.semi_finals.get(&0), Some(&1));
        assert_eq!(team_0_slots.final_match, 1);

        // Check Team 16's bracket slot stats
        // R32: match 8 (Team 16 vs Team 17) -> slot 8
        // R16: match 4 (Team 16 vs Team 18) -> slot 4
        // QF: match 2 (Team 16 vs Team 20) -> slot 2
        // SF: match 1 (Team 16 vs Team 24) -> slot 1
        // Final: Team 16 vs Team 0
        let team_16_slots = aggregated.bracket_slot_stats.get(&TeamId(16)).unwrap();
        assert_eq!(team_16_slots.round_of_32.get(&8), Some(&1));
        assert_eq!(team_16_slots.round_of_16.get(&4), Some(&1));
        assert_eq!(team_16_slots.quarter_finals.get(&2), Some(&1));
        assert_eq!(team_16_slots.semi_finals.get(&1), Some(&1));
        assert_eq!(team_16_slots.final_match, 1);

        // Check Team 1 (eliminated in R32)
        // R32: match 0 (Team 0 vs Team 1) -> slot 0
        let team_1_slots = aggregated.bracket_slot_stats.get(&TeamId(1)).unwrap();
        assert_eq!(team_1_slots.round_of_32.get(&0), Some(&1));
        assert!(team_1_slots.round_of_16.is_empty());
        assert!(team_1_slots.quarter_finals.is_empty());
        assert!(team_1_slots.semi_finals.is_empty());
        assert_eq!(team_1_slots.final_match, 0);
    }

    #[test]
    fn test_bracket_slot_stats_serialization() {
        use crate::runner::{SimulationConfig, SimulationRunner};
        use wc_strategies::EloStrategy;

        let tournament = create_test_tournament();
        let strategy = EloStrategy::default();
        let config = SimulationConfig::with_iterations(10).with_seed(42);
        let runner = SimulationRunner::new(&tournament, &strategy, config);
        let results = runner.run_with_progress(|_, _| {});

        let json = serde_json::to_string(&results).unwrap();
        assert!(json.contains("bracket_slot_stats"), "bracket_slot_stats should be in JSON output");
    }
}
