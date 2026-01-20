//! Aggregation of simulation results.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use wc_core::{MatchResult, TeamId, Tournament, TournamentResult};

use crate::optimal_bracket::compute_optimal_bracket;
use crate::path_tracker::{BracketSlotStats, BracketSlotWinStats, MostFrequentBracket, MostLikelyBracket, MostLikelyBracketSlot, OptimalBracket, PathStatistics, SlotOpponentStats};

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
    /// Bracket slot WIN statistics for each team (only winners, not participants)
    pub bracket_slot_win_stats: HashMap<TeamId, BracketSlotWinStats>,
    /// Slot-specific opponent statistics (who did they face in each specific slot)
    pub slot_opponent_stats: HashMap<TeamId, SlotOpponentStats>,
    /// The most frequently occurring complete bracket outcome
    pub most_frequent_bracket: Option<MostFrequentBracket>,
    /// The most likely bracket computed via greedy algorithm (ensures unique teams)
    pub most_likely_bracket: MostLikelyBracket,
    /// The optimal bracket computed via Hungarian algorithm (guarantees exactly 32 unique teams)
    pub optimal_bracket: OptimalBracket,
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

        let mut slot_opponent_stats: HashMap<TeamId, SlotOpponentStats> = HashMap::new();
        let mut bracket_slot_win_stats: HashMap<TeamId, BracketSlotWinStats> = HashMap::new();

        // Track complete bracket outcomes for finding the most frequent bracket
        // Key: signature string of winner IDs, Value: (count, first KnockoutBracket that produced it)
        let mut bracket_outcomes: HashMap<String, (u32, wc_core::KnockoutBracket)> = HashMap::new();
        const MAX_UNIQUE_BRACKETS: usize = 1000; // Limit memory usage

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
            bracket_slot_win_stats.insert(team.id, BracketSlotWinStats::new());
            slot_opponent_stats.insert(team.id, SlotOpponentStats::new());
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

            // Track complete bracket outcome
            // Create signature from all match winners
            let bracket_sig = create_bracket_signature(&result.knockout_bracket);
            if bracket_outcomes.len() < MAX_UNIQUE_BRACKETS || bracket_outcomes.contains_key(&bracket_sig) {
                bracket_outcomes
                    .entry(bracket_sig)
                    .and_modify(|(count, _)| *count += 1)
                    .or_insert((1, result.knockout_bracket.clone()));
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
                let slot_u8 = slot as u8;
                if let Some(stats) = bracket_slot_stats.get_mut(&m.home_team) {
                    stats.record_slot("round_of_32", slot_u8);
                }
                if let Some(stats) = bracket_slot_stats.get_mut(&m.away_team) {
                    stats.record_slot("round_of_32", slot_u8);
                }
                // Track slot-specific opponents
                if let Some(stats) = slot_opponent_stats.get_mut(&m.home_team) {
                    stats.record_opponent("round_of_32", slot_u8, m.away_team);
                }
                if let Some(stats) = slot_opponent_stats.get_mut(&m.away_team) {
                    stats.record_opponent("round_of_32", slot_u8, m.home_team);
                }
            }

            // Round of 16: slots 0-7
            for (slot, m) in result.knockout_bracket.round_of_16.iter().enumerate() {
                let slot_u8 = slot as u8;
                if let Some(stats) = bracket_slot_stats.get_mut(&m.home_team) {
                    stats.record_slot("round_of_16", slot_u8);
                }
                if let Some(stats) = bracket_slot_stats.get_mut(&m.away_team) {
                    stats.record_slot("round_of_16", slot_u8);
                }
                // Track slot-specific opponents
                if let Some(stats) = slot_opponent_stats.get_mut(&m.home_team) {
                    stats.record_opponent("round_of_16", slot_u8, m.away_team);
                }
                if let Some(stats) = slot_opponent_stats.get_mut(&m.away_team) {
                    stats.record_opponent("round_of_16", slot_u8, m.home_team);
                }
            }

            // Quarter-finals: slots 0-3
            for (slot, m) in result.knockout_bracket.quarter_finals.iter().enumerate() {
                let slot_u8 = slot as u8;
                if let Some(stats) = bracket_slot_stats.get_mut(&m.home_team) {
                    stats.record_slot("quarter_finals", slot_u8);
                }
                if let Some(stats) = bracket_slot_stats.get_mut(&m.away_team) {
                    stats.record_slot("quarter_finals", slot_u8);
                }
                // Track slot-specific opponents
                if let Some(stats) = slot_opponent_stats.get_mut(&m.home_team) {
                    stats.record_opponent("quarter_finals", slot_u8, m.away_team);
                }
                if let Some(stats) = slot_opponent_stats.get_mut(&m.away_team) {
                    stats.record_opponent("quarter_finals", slot_u8, m.home_team);
                }
            }

            // Semi-finals: slots 0-1
            for (slot, m) in result.knockout_bracket.semi_finals.iter().enumerate() {
                let slot_u8 = slot as u8;
                if let Some(stats) = bracket_slot_stats.get_mut(&m.home_team) {
                    stats.record_slot("semi_finals", slot_u8);
                }
                if let Some(stats) = bracket_slot_stats.get_mut(&m.away_team) {
                    stats.record_slot("semi_finals", slot_u8);
                }
                // Track slot-specific opponents
                if let Some(stats) = slot_opponent_stats.get_mut(&m.home_team) {
                    stats.record_opponent("semi_finals", slot_u8, m.away_team);
                }
                if let Some(stats) = slot_opponent_stats.get_mut(&m.away_team) {
                    stats.record_opponent("semi_finals", slot_u8, m.home_team);
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
                // Track slot-specific opponents for final
                if let Some(stats) = slot_opponent_stats.get_mut(&m.home_team) {
                    stats.record_final_opponent(m.away_team);
                }
                if let Some(stats) = slot_opponent_stats.get_mut(&m.away_team) {
                    stats.record_final_opponent(m.home_team);
                }
            }

            // Track WINS per slot (only the winner, not both participants)
            // Round of 32: slots 0-15
            for (slot, m) in result.knockout_bracket.round_of_32.iter().enumerate() {
                if let Some(winner) = m.winner() {
                    if let Some(stats) = bracket_slot_win_stats.get_mut(&winner) {
                        stats.record_win("round_of_32", slot as u8);
                    }
                }
            }

            // Round of 16: slots 0-7
            for (slot, m) in result.knockout_bracket.round_of_16.iter().enumerate() {
                if let Some(winner) = m.winner() {
                    if let Some(stats) = bracket_slot_win_stats.get_mut(&winner) {
                        stats.record_win("round_of_16", slot as u8);
                    }
                }
            }

            // Quarter-finals: slots 0-3
            for (slot, m) in result.knockout_bracket.quarter_finals.iter().enumerate() {
                if let Some(winner) = m.winner() {
                    if let Some(stats) = bracket_slot_win_stats.get_mut(&winner) {
                        stats.record_win("quarter_finals", slot as u8);
                    }
                }
            }

            // Semi-finals: slots 0-1
            for (slot, m) in result.knockout_bracket.semi_finals.iter().enumerate() {
                if let Some(winner) = m.winner() {
                    if let Some(stats) = bracket_slot_win_stats.get_mut(&winner) {
                        stats.record_win("semi_finals", slot as u8);
                    }
                }
            }

            // Final: single match
            if let Some(winner) = result.knockout_bracket.final_match.winner() {
                if let Some(stats) = bracket_slot_win_stats.get_mut(&winner) {
                    stats.record_win("final", 0);
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

        // Find the most frequent complete bracket outcome WHERE the champion is the most_likely_winner
        // This ensures the bracket display is consistent with the overall winner shown in results
        let most_frequent_bracket = {
            // Filter to brackets where champion matches most_likely_winner
            let matching_brackets: Vec<_> = bracket_outcomes
                .into_iter()
                .filter(|(_, (_, bracket))| {
                    bracket.final_match.winner() == Some(most_likely_winner)
                })
                .collect();

            // Find the most frequent among matching brackets
            if let Some((_, (count, bracket))) = matching_brackets
                .into_iter()
                .max_by_key(|(_, (count, _))| *count)
            {
                // Extract winner IDs from each round
                let r32_winners: Vec<TeamId> = bracket
                    .round_of_32
                    .iter()
                    .filter_map(|m| m.winner())
                    .collect();
                let r16_winners: Vec<TeamId> = bracket
                    .round_of_16
                    .iter()
                    .filter_map(|m| m.winner())
                    .collect();
                let qf_winners: Vec<TeamId> = bracket
                    .quarter_finals
                    .iter()
                    .filter_map(|m| m.winner())
                    .collect();
                let sf_winners: Vec<TeamId> = bracket
                    .semi_finals
                    .iter()
                    .filter_map(|m| m.winner())
                    .collect();

                Some(MostFrequentBracket {
                    count,
                    probability: count as f64 / total as f64,
                    round_of_32_winners: r32_winners,
                    round_of_16_winners: r16_winners,
                    quarter_final_winners: qf_winners,
                    semi_final_winners: sf_winners,
                    champion: most_likely_winner,
                })
            } else {
                None
            }
        };

        // Compute the most likely bracket using greedy algorithm
        let most_likely_bracket = compute_greedy_bracket(
            &team_stats,
            &bracket_slot_win_stats,
            &bracket_slot_stats,
            total,
        );

        // Compute the optimal bracket using Hungarian algorithm
        let team_ids: Vec<wc_core::TeamId> = tournament.teams.iter().map(|t| t.id).collect();
        let optimal_bracket = compute_optimal_bracket(
            &team_ids,
            &tournament.groups,
            &bracket_slot_stats,
            &bracket_slot_win_stats,
            total,
        );

        Self {
            total_simulations: total,
            team_stats,
            most_likely_winner,
            most_likely_final,
            path_stats,
            bracket_slot_stats,
            bracket_slot_win_stats,
            slot_opponent_stats,
            most_frequent_bracket,
            most_likely_bracket,
            optimal_bracket,
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

/// Compute the most likely bracket using a greedy algorithm.
///
/// The algorithm ensures:
/// 1. Each team appears at most once (no duplicates)
/// 2. Tournament structure is valid (later round winners must have won feeder matches)
/// 3. Higher-ranked teams get priority for their best slots
///
/// It uses a participation fallback when a team has no wins at a slot - this fixes
/// the issue where teams that qualified for knockouts but lost every R32 match
/// would be missing from the bracket entirely.
fn compute_greedy_bracket(
    team_stats: &HashMap<TeamId, TeamStatistics>,
    bracket_slot_win_stats: &HashMap<TeamId, BracketSlotWinStats>,
    bracket_slot_stats: &HashMap<TeamId, BracketSlotStats>,
    total_simulations: u32,
) -> MostLikelyBracket {
    use std::collections::HashSet;

    // Sort teams by champion count (win probability) descending
    let mut teams_by_win_prob: Vec<_> = team_stats.iter().collect();
    teams_by_win_prob.sort_by(|a, b| b.1.champion.cmp(&a.1.champion));

    // Helper: get count for a team at a specific round/slot.
    // First tries win stats, then falls back to participation stats.
    let get_count = |team_id: TeamId, round: &str, slot: u8| -> u32 {
        // Try win stats first
        if let Some(ws) = bracket_slot_win_stats.get(&team_id) {
            let win_count = match round {
                "round_of_32" => ws.round_of_32.get(&slot).copied().unwrap_or(0),
                "round_of_16" => ws.round_of_16.get(&slot).copied().unwrap_or(0),
                "quarter_finals" => ws.quarter_finals.get(&slot).copied().unwrap_or(0),
                "semi_finals" => ws.semi_finals.get(&slot).copied().unwrap_or(0),
                "final" => ws.final_match,
                _ => 0,
            };
            if win_count > 0 {
                return win_count;
            }
        }
        // Fallback to participation stats (team played but lost)
        if let Some(ps) = bracket_slot_stats.get(&team_id) {
            match round {
                "round_of_32" => ps.round_of_32.get(&slot).copied().unwrap_or(0),
                "round_of_16" => ps.round_of_16.get(&slot).copied().unwrap_or(0),
                "quarter_finals" => ps.quarter_finals.get(&slot).copied().unwrap_or(0),
                "semi_finals" => ps.semi_finals.get(&slot).copied().unwrap_or(0),
                "final" => ps.final_match,
                _ => 0,
            }
        } else {
            0
        }
    };

    // Helper to create slot data
    let make_slot_data = |team_id: TeamId, count: u32| -> MostLikelyBracketSlot {
        MostLikelyBracketSlot {
            team_id,
            count,
            probability: count as f64 / total_simulations as f64,
        }
    };

    // Phase 1: Assign R32 slots greedily by team win probability
    let mut r32: HashMap<u8, TeamId> = HashMap::new();
    let mut used_in_r32: HashSet<TeamId> = HashSet::new();

    for (team_id, stats) in &teams_by_win_prob {
        if used_in_r32.contains(team_id) {
            continue;
        }
        // Skip teams that never reached R32
        if stats.reached_round_of_32 == 0 {
            continue;
        }

        // Find best available R32 slot for this team
        let mut best_slot: Option<u8> = None;
        let mut best_count = 0u32;
        for slot in 0..16u8 {
            if r32.contains_key(&slot) {
                continue; // Slot already taken
            }
            let count = get_count(**team_id, "round_of_32", slot);
            if count > best_count {
                best_count = count;
                best_slot = Some(slot);
            }
        }

        if let Some(slot) = best_slot {
            if best_count > 0 {
                r32.insert(slot, **team_id);
                used_in_r32.insert(**team_id);
            }
        }
    }

    // Phase 2: Assign R16 based on R32 feeder slots
    // R16 slot i receives winners from R32 slots 2i and 2i+1
    let mut r16: HashMap<u8, TeamId> = HashMap::new();
    for slot in 0..8u8 {
        let feeders = [slot * 2, slot * 2 + 1];
        let mut best_team: Option<TeamId> = None;
        let mut best_count = 0u32;
        for feeder in feeders {
            if let Some(&team_id) = r32.get(&feeder) {
                let count = get_count(team_id, "round_of_16", slot);
                if count > best_count {
                    best_count = count;
                    best_team = Some(team_id);
                }
            }
        }
        if let Some(team_id) = best_team {
            if best_count > 0 {
                r16.insert(slot, team_id);
            }
        }
    }

    // Phase 3: Assign QF based on R16 feeder slots
    let mut qf: HashMap<u8, TeamId> = HashMap::new();
    for slot in 0..4u8 {
        let feeders = [slot * 2, slot * 2 + 1];
        let mut best_team: Option<TeamId> = None;
        let mut best_count = 0u32;
        for feeder in feeders {
            if let Some(&team_id) = r16.get(&feeder) {
                let count = get_count(team_id, "quarter_finals", slot);
                if count > best_count {
                    best_count = count;
                    best_team = Some(team_id);
                }
            }
        }
        if let Some(team_id) = best_team {
            if best_count > 0 {
                qf.insert(slot, team_id);
            }
        }
    }

    // Phase 4: Assign SF based on QF feeder slots
    let mut sf: HashMap<u8, TeamId> = HashMap::new();
    for slot in 0..2u8 {
        let feeders = [slot * 2, slot * 2 + 1];
        let mut best_team: Option<TeamId> = None;
        let mut best_count = 0u32;
        for feeder in feeders {
            if let Some(&team_id) = qf.get(&feeder) {
                let count = get_count(team_id, "semi_finals", slot);
                if count > best_count {
                    best_count = count;
                    best_team = Some(team_id);
                }
            }
        }
        if let Some(team_id) = best_team {
            if best_count > 0 {
                sf.insert(slot, team_id);
            }
        }
    }

    // Phase 5: Assign Final based on SF feeder slots
    let mut final_winner: Option<(TeamId, u32)> = None;
    {
        let feeders = [0u8, 1u8];
        let mut best_team: Option<TeamId> = None;
        let mut best_count = 0u32;
        for feeder in feeders {
            if let Some(&team_id) = sf.get(&feeder) {
                let count = get_count(team_id, "final", 0);
                if count > best_count {
                    best_count = count;
                    best_team = Some(team_id);
                }
            }
        }
        if let Some(team_id) = best_team {
            if best_count > 0 {
                final_winner = Some((team_id, best_count));
            }
        }
    }

    // Build result with slot data including counts and probabilities
    let mut result_r32: HashMap<u8, MostLikelyBracketSlot> = HashMap::new();
    for (slot, team_id) in &r32 {
        let count = get_count(*team_id, "round_of_32", *slot);
        result_r32.insert(*slot, make_slot_data(*team_id, count));
    }

    let mut result_r16: HashMap<u8, MostLikelyBracketSlot> = HashMap::new();
    for (slot, team_id) in &r16 {
        let count = get_count(*team_id, "round_of_16", *slot);
        result_r16.insert(*slot, make_slot_data(*team_id, count));
    }

    let mut result_qf: HashMap<u8, MostLikelyBracketSlot> = HashMap::new();
    for (slot, team_id) in &qf {
        let count = get_count(*team_id, "quarter_finals", *slot);
        result_qf.insert(*slot, make_slot_data(*team_id, count));
    }

    let mut result_sf: HashMap<u8, MostLikelyBracketSlot> = HashMap::new();
    for (slot, team_id) in &sf {
        let count = get_count(*team_id, "semi_finals", *slot);
        result_sf.insert(*slot, make_slot_data(*team_id, count));
    }

    let final_match = final_winner.map(|(team_id, count)| make_slot_data(team_id, count));
    let champion = final_match.clone();

    MostLikelyBracket {
        round_of_32: result_r32,
        round_of_16: result_r16,
        quarter_finals: result_qf,
        semi_finals: result_sf,
        final_match,
        champion,
    }
}

/// Create a unique signature string for a complete bracket outcome.
/// The signature is based on all match winners in order.
fn create_bracket_signature(bracket: &wc_core::KnockoutBracket) -> String {
    let mut parts = Vec::new();

    // R32 winners (16 matches)
    for m in &bracket.round_of_32 {
        if let Some(winner) = m.winner() {
            parts.push(winner.0.to_string());
        }
    }

    // R16 winners (8 matches)
    for m in &bracket.round_of_16 {
        if let Some(winner) = m.winner() {
            parts.push(winner.0.to_string());
        }
    }

    // QF winners (4 matches)
    for m in &bracket.quarter_finals {
        if let Some(winner) = m.winner() {
            parts.push(winner.0.to_string());
        }
    }

    // SF winners (2 matches)
    for m in &bracket.semi_finals {
        if let Some(winner) = m.winner() {
            parts.push(winner.0.to_string());
        }
    }

    // Final winner (champion)
    if let Some(champion) = bracket.final_match.winner() {
        parts.push(champion.0.to_string());
    }

    parts.join("-")
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
                let group_id = GroupId::from_index(i as u8);
                GroupResult {
                    group_id,
                    matches: vec![],
                    standings: vec![
                        GroupStanding::new(TeamId(start), group_id),
                        GroupStanding::new(TeamId(start + 1), group_id),
                        GroupStanding::new(TeamId(start + 2), group_id),
                        GroupStanding::new(TeamId(start + 3), group_id),
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

    #[test]
    fn test_most_frequent_bracket() {
        use crate::runner::{SimulationConfig, SimulationRunner};
        use wc_strategies::EloStrategy;

        let tournament = create_test_tournament();
        let strategy = EloStrategy::default();
        let config = SimulationConfig::with_iterations(100).with_seed(42);
        let runner = SimulationRunner::new(&tournament, &strategy, config);
        let results = runner.run_with_progress(|_, _| {});

        // Should have most_frequent_bracket populated
        assert!(results.most_frequent_bracket.is_some(), "most_frequent_bracket should be Some");

        let bracket = results.most_frequent_bracket.as_ref().unwrap();
        println!("Most frequent bracket count: {}", bracket.count);
        println!("Probability: {:.4}", bracket.probability);
        println!("Champion: {:?}", bracket.champion);
        println!("R32 winners: {:?}", bracket.round_of_32_winners);

        // Verify structure
        assert_eq!(bracket.round_of_32_winners.len(), 16, "Should have 16 R32 winners");
        assert_eq!(bracket.round_of_16_winners.len(), 8, "Should have 8 R16 winners");
        assert_eq!(bracket.quarter_final_winners.len(), 4, "Should have 4 QF winners");
        assert_eq!(bracket.semi_final_winners.len(), 2, "Should have 2 SF winners");

        // Verify champion matches most_likely_winner
        assert_eq!(
            bracket.champion, results.most_likely_winner,
            "Bracket champion should match most_likely_winner"
        );

        // Verify serialization includes the field
        let json = serde_json::to_string(&results).unwrap();
        assert!(json.contains("most_frequent_bracket"), "most_frequent_bracket should be in JSON");
        assert!(json.contains("round_of_32_winners"), "round_of_32_winners should be in JSON");
    }

    #[test]
    fn test_bracket_slot_win_stats_tracking() {
        let tournament = create_test_tournament();
        let results = vec![create_dummy_tournament_result()];

        let aggregated = AggregatedResults::from_results(results, &tournament);

        // bracket_slot_win_stats should be populated for all teams
        assert_eq!(aggregated.bracket_slot_win_stats.len(), 48);

        // Check that only WINNERS are recorded, not losers
        // In the dummy result:
        // R32 match 0: Team 0 beats Team 1 (1-0)
        // So Team 0 should have a win at slot 0, Team 1 should have NO wins

        // Team 0 (champion) should have wins at every slot they played
        let team_0_wins = aggregated.bracket_slot_win_stats.get(&TeamId(0)).unwrap();
        assert_eq!(team_0_wins.round_of_32.get(&0), Some(&1), "Team 0 should win R32 slot 0");
        assert_eq!(team_0_wins.round_of_16.get(&0), Some(&1), "Team 0 should win R16 slot 0");
        assert_eq!(team_0_wins.quarter_finals.get(&0), Some(&1), "Team 0 should win QF slot 0");
        assert_eq!(team_0_wins.semi_finals.get(&0), Some(&1), "Team 0 should win SF slot 0");
        assert_eq!(team_0_wins.final_match, 1, "Team 0 should win the final");

        // Team 1 (eliminated in R32) should have NO wins
        let team_1_wins = aggregated.bracket_slot_win_stats.get(&TeamId(1)).unwrap();
        assert!(team_1_wins.round_of_32.is_empty(), "Team 1 should have no R32 wins");
        assert!(team_1_wins.round_of_16.is_empty(), "Team 1 should have no R16 wins");
        assert!(team_1_wins.quarter_finals.is_empty(), "Team 1 should have no QF wins");
        assert!(team_1_wins.semi_finals.is_empty(), "Team 1 should have no SF wins");
        assert_eq!(team_1_wins.final_match, 0, "Team 1 should not win the final");

        // Team 16 (runner-up) should have wins in earlier rounds but not the final
        let team_16_wins = aggregated.bracket_slot_win_stats.get(&TeamId(16)).unwrap();
        assert_eq!(team_16_wins.round_of_32.get(&8), Some(&1), "Team 16 should win R32 slot 8");
        assert_eq!(team_16_wins.round_of_16.get(&4), Some(&1), "Team 16 should win R16 slot 4");
        assert_eq!(team_16_wins.quarter_finals.get(&2), Some(&1), "Team 16 should win QF slot 2");
        assert_eq!(team_16_wins.semi_finals.get(&1), Some(&1), "Team 16 should win SF slot 1");
        assert_eq!(team_16_wins.final_match, 0, "Team 16 should NOT win the final (runner-up)");
    }

    #[test]
    fn test_win_stats_less_than_or_equal_participation_stats() {
        use crate::runner::{SimulationConfig, SimulationRunner};
        use wc_strategies::EloStrategy;

        let tournament = create_test_tournament();
        let strategy = EloStrategy::default();
        let config = SimulationConfig::with_iterations(100).with_seed(42);
        let runner = SimulationRunner::new(&tournament, &strategy, config);
        let results = runner.run_with_progress(|_, _| {});

        // For every team, wins <= participation for all slots
        for (team_id, participation_stats) in &results.bracket_slot_stats {
            let win_stats = results.bracket_slot_win_stats.get(team_id).unwrap();

            // R32 slots
            for (slot, &participation_count) in &participation_stats.round_of_32 {
                let win_count = win_stats.round_of_32.get(slot).unwrap_or(&0);
                assert!(
                    *win_count <= participation_count,
                    "Team {:?} R32 slot {}: wins ({}) > participation ({})",
                    team_id, slot, win_count, participation_count
                );
            }

            // R16 slots
            for (slot, &participation_count) in &participation_stats.round_of_16 {
                let win_count = win_stats.round_of_16.get(slot).unwrap_or(&0);
                assert!(
                    *win_count <= participation_count,
                    "Team {:?} R16 slot {}: wins ({}) > participation ({})",
                    team_id, slot, win_count, participation_count
                );
            }

            // QF slots
            for (slot, &participation_count) in &participation_stats.quarter_finals {
                let win_count = win_stats.quarter_finals.get(slot).unwrap_or(&0);
                assert!(
                    *win_count <= participation_count,
                    "Team {:?} QF slot {}: wins ({}) > participation ({})",
                    team_id, slot, win_count, participation_count
                );
            }

            // SF slots
            for (slot, &participation_count) in &participation_stats.semi_finals {
                let win_count = win_stats.semi_finals.get(slot).unwrap_or(&0);
                assert!(
                    *win_count <= participation_count,
                    "Team {:?} SF slot {}: wins ({}) > participation ({})",
                    team_id, slot, win_count, participation_count
                );
            }

            // Final
            assert!(
                win_stats.final_match <= participation_stats.final_match,
                "Team {:?} Final: wins ({}) > participation ({})",
                team_id, win_stats.final_match, participation_stats.final_match
            );
        }
    }

    #[test]
    fn test_bracket_slot_win_stats_serialization() {
        use crate::runner::{SimulationConfig, SimulationRunner};
        use wc_strategies::EloStrategy;

        let tournament = create_test_tournament();
        let strategy = EloStrategy::default();
        let config = SimulationConfig::with_iterations(10).with_seed(42);
        let runner = SimulationRunner::new(&tournament, &strategy, config);
        let results = runner.run_with_progress(|_, _| {});

        let json = serde_json::to_string(&results).unwrap();
        assert!(json.contains("bracket_slot_win_stats"), "bracket_slot_win_stats should be in JSON output");
    }

    #[test]
    fn test_most_likely_bracket_computed() {
        use crate::runner::{SimulationConfig, SimulationRunner};
        use wc_strategies::EloStrategy;

        let tournament = create_test_tournament();
        let strategy = EloStrategy::default();
        let config = SimulationConfig::with_iterations(100).with_seed(42);
        let runner = SimulationRunner::new(&tournament, &strategy, config);
        let results = runner.run_with_progress(|_, _| {});

        let bracket = &results.most_likely_bracket;

        // Verify structure - should have some entries in each round
        // Note: not all slots may be filled depending on simulation results
        assert!(!bracket.round_of_32.is_empty(), "R32 should have entries");
        assert!(!bracket.round_of_16.is_empty(), "R16 should have entries");
        assert!(!bracket.quarter_finals.is_empty(), "QF should have entries");
        assert!(!bracket.semi_finals.is_empty(), "SF should have entries");
        assert!(bracket.final_match.is_some(), "Final should have a winner");
        assert!(bracket.champion.is_some(), "Champion should be set");

        // Verify no duplicate teams in R32
        let r32_teams: std::collections::HashSet<_> = bracket.round_of_32.values()
            .map(|slot| slot.team_id)
            .collect();
        assert_eq!(r32_teams.len(), bracket.round_of_32.len(), "No duplicate teams in R32");

        // Verify bracket structure consistency: R16 winners must be from R32 feeders
        for (slot, r16_data) in &bracket.round_of_16 {
            let slot_num = *slot;
            let feeder1 = slot_num * 2;
            let feeder2 = slot_num * 2 + 1;
            let r32_team1 = bracket.round_of_32.get(&feeder1).map(|s| s.team_id);
            let r32_team2 = bracket.round_of_32.get(&feeder2).map(|s| s.team_id);
            assert!(
                r32_team1 == Some(r16_data.team_id) || r32_team2 == Some(r16_data.team_id),
                "R16 slot {} winner {:?} must be from R32 feeders {:?} or {:?}",
                slot, r16_data.team_id, r32_team1, r32_team2
            );
        }

        // Verify serialization includes the field
        let json = serde_json::to_string(&results).unwrap();
        assert!(json.contains("most_likely_bracket"), "most_likely_bracket should be in JSON");
    }

    #[test]
    fn test_most_likely_bracket_uses_participation_fallback() {
        let tournament = create_test_tournament();
        let results = vec![create_dummy_tournament_result()];

        let aggregated = AggregatedResults::from_results(results, &tournament);

        // In the dummy result, Team 1 lost in R32 (no wins) but participated
        // The greedy algorithm should still be able to assign them using participation stats
        // if they are ranked highly enough

        // Check that the bracket was computed
        let bracket = &aggregated.most_likely_bracket;

        // Verify Team 0 (champion with wins) is in the bracket
        let r32_teams: Vec<_> = bracket.round_of_32.values()
            .map(|slot| slot.team_id)
            .collect();
        assert!(r32_teams.contains(&TeamId(0)), "Champion Team 0 should be in R32");

        // Verify all probabilities are valid (0 <= p <= 1)
        for slot in bracket.round_of_32.values() {
            assert!(slot.probability >= 0.0 && slot.probability <= 1.0,
                "Probability should be between 0 and 1");
        }
    }
}
