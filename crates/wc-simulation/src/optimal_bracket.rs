//! Optimal bracket computation using the Hungarian algorithm.
//!
//! This module computes a valid "most likely bracket" where:
//! - All 32 R32 positions are filled with exactly 32 unique teams
//! - Each team respects FIFA bracket eligibility constraints
//! - Total probability is maximized using the Kuhn-Munkres algorithm

use std::collections::{HashMap, HashSet};

use pathfinding::kuhn_munkres::kuhn_munkres;
use pathfinding::matrix::Matrix;
use wc_core::bracket::{SlotSource, R32_BRACKET};
use wc_core::{Group, TeamId};

use crate::path_tracker::{
    BracketSlotStats, BracketSlotWinStats, MostLikelyBracketSlot, OptimalBracket, OptimalR32Match,
};

/// Pool of possible source groups for each third-place bracket slot.
/// Index order: [1E, 1I, 1A, 1L, 1D, 1G, 1B, 1K]
const THIRD_PLACE_POOLS: [[char; 5]; 8] = [
    ['A', 'B', 'C', 'D', 'F'], // slot 0: 1E plays 3rd from A/B/C/D/F
    ['C', 'D', 'F', 'G', 'H'], // slot 1: 1I plays 3rd from C/D/F/G/H
    ['C', 'E', 'F', 'H', 'I'], // slot 2: 1A plays 3rd from C/E/F/H/I
    ['E', 'H', 'I', 'J', 'K'], // slot 3: 1L plays 3rd from E/H/I/J/K
    ['B', 'E', 'F', 'I', 'J'], // slot 4: 1D plays 3rd from B/E/F/I/J
    ['A', 'E', 'H', 'I', 'J'], // slot 5: 1G plays 3rd from A/E/H/I/J
    ['E', 'F', 'G', 'I', 'J'], // slot 6: 1B plays 3rd from E/F/G/I/J
    ['D', 'E', 'I', 'J', 'L'], // slot 7: 1K plays 3rd from D/E/I/J/L
];

/// R32 position: (match_slot, side) where side is 0=team_a, 1=team_b
type R32Position = (u8, u8);

/// Eligibility map: team_id -> list of (R32 match slot, side)
type EligibilityMap = HashMap<TeamId, Vec<R32Position>>;

/// Build a mapping from team_id to their group letter.
fn build_team_to_group_map(groups: &[Group]) -> HashMap<TeamId, char> {
    let mut map = HashMap::new();
    for group in groups {
        for &team_id in &group.teams {
            map.insert(team_id, group.id.0);
        }
    }
    map
}

/// Check if a team from a given group can be placed at a SlotSource position.
fn team_can_be_at_source(team_group: char, source: &SlotSource) -> bool {
    match source {
        SlotSource::GroupTeam { group, position: _ } => team_group == *group,
        SlotSource::ThirdPlacePool { slot_index } => {
            let pool = &THIRD_PLACE_POOLS[*slot_index as usize];
            pool.contains(&team_group)
        }
    }
}

/// Build the eligibility map: for each team, determine all valid R32 positions.
fn build_eligibility_map(
    teams: &[TeamId],
    team_to_group: &HashMap<TeamId, char>,
) -> EligibilityMap {
    let mut eligibility: EligibilityMap = HashMap::new();

    for &team_id in teams {
        let team_group = match team_to_group.get(&team_id) {
            Some(g) => *g,
            None => continue,
        };

        let mut valid_positions = Vec::new();

        for (slot_idx, r32_match) in R32_BRACKET.iter().enumerate() {
            let slot = slot_idx as u8;

            // Check team_a position
            if team_can_be_at_source(team_group, &r32_match.team_a) {
                valid_positions.push((slot, 0));
            }

            // Check team_b position
            if team_can_be_at_source(team_group, &r32_match.team_b) {
                valid_positions.push((slot, 1));
            }
        }

        if !valid_positions.is_empty() {
            eligibility.insert(team_id, valid_positions);
        }
    }

    eligibility
}

/// Convert R32 position (slot, side) to a linear index (0-31).
fn position_to_index(slot: u8, side: u8) -> usize {
    (slot as usize) * 2 + (side as usize)
}

/// Convert linear index (0-31) to R32 position (slot, side).
fn index_to_position(idx: usize) -> (u8, u8) {
    let slot = (idx / 2) as u8;
    let side = (idx % 2) as u8;
    (slot, side)
}

/// Get the participation count for a team at a given R32 slot.
/// Uses win stats first, then falls back to participation stats.
fn get_participation_count(
    team_id: TeamId,
    slot: u8,
    bracket_slot_stats: &HashMap<TeamId, BracketSlotStats>,
    bracket_slot_win_stats: &HashMap<TeamId, BracketSlotWinStats>,
) -> u32 {
    // Try win stats first (more accurate for "most likely winner")
    if let Some(win_stats) = bracket_slot_win_stats.get(&team_id) {
        if let Some(&count) = win_stats.round_of_32.get(&slot) {
            if count > 0 {
                return count;
            }
        }
    }

    // Fall back to participation stats
    if let Some(part_stats) = bracket_slot_stats.get(&team_id) {
        return part_stats.round_of_32.get(&slot).copied().unwrap_or(0);
    }

    0
}

/// Compute the optimal bracket using the Hungarian algorithm.
///
/// This assigns 32 teams to 32 R32 positions (16 matches × 2 sides)
/// to maximize total participation probability while respecting eligibility.
pub fn compute_optimal_bracket(
    teams: &[TeamId],
    groups: &[Group],
    bracket_slot_stats: &HashMap<TeamId, BracketSlotStats>,
    bracket_slot_win_stats: &HashMap<TeamId, BracketSlotWinStats>,
    total_simulations: u32,
) -> OptimalBracket {
    let team_to_group = build_team_to_group_map(groups);
    let eligibility = build_eligibility_map(teams, &team_to_group);

    // Filter to teams that have any eligibility
    let eligible_teams: Vec<TeamId> = teams
        .iter()
        .filter(|t| eligibility.contains_key(t))
        .copied()
        .collect();

    let num_teams = eligible_teams.len();
    let num_positions = 32; // 16 matches × 2 sides

    // Build cost matrix for Hungarian algorithm
    // We need a square matrix, so pad to max(num_teams, num_positions)
    let matrix_size = num_teams.max(num_positions);

    // Build the matrix: rows = teams, cols = R32 positions
    let mut weights = vec![vec![0i64; matrix_size]; matrix_size];

    for (team_idx, &team_id) in eligible_teams.iter().enumerate() {
        if let Some(valid_positions) = eligibility.get(&team_id) {
            for &(slot, side) in valid_positions {
                let pos_idx = position_to_index(slot, side);
                let count = get_participation_count(
                    team_id,
                    slot,
                    bracket_slot_stats,
                    bracket_slot_win_stats,
                ) as i64;

                // Use count directly (kuhn_munkres maximizes)
                weights[team_idx][pos_idx] = count;
            }
        }
        // Ineligible positions remain 0 (worst choice for maximization)
    }

    // Pad extra rows/cols with 0 if needed
    // (already initialized to 0)

    // Convert to pathfinding Matrix
    let matrix = Matrix::from_rows(weights).expect("Failed to create matrix");

    // Run Hungarian algorithm (kuhn_munkres finds maximum weight matching)
    let (_, assignment) = kuhn_munkres(&matrix);

    // Extract the assignment: team_idx -> position_idx
    let mut position_assignments: HashMap<usize, TeamId> = HashMap::new();

    for (team_idx, &pos_idx) in assignment.iter().enumerate() {
        if team_idx < eligible_teams.len() && pos_idx < num_positions {
            let team_id = eligible_teams[team_idx];

            // Verify this is a valid position for the team
            if let Some(valid_positions) = eligibility.get(&team_id) {
                let (slot, side) = index_to_position(pos_idx);
                if valid_positions.contains(&(slot, side)) {
                    position_assignments.insert(pos_idx, team_id);
                }
            }
        }
    }

    // Build R32 matches from assignments
    let mut r32_matches: Vec<OptimalR32Match> = Vec::with_capacity(16);

    for slot in 0..16u8 {
        let pos_a = position_to_index(slot, 0);
        let pos_b = position_to_index(slot, 1);

        let team_a_id = position_assignments.get(&pos_a).copied();
        let team_b_id = position_assignments.get(&pos_b).copied();

        match (team_a_id, team_b_id) {
            (Some(a_id), Some(b_id)) => {
                let a_count = get_participation_count(
                    a_id,
                    slot,
                    bracket_slot_stats,
                    bracket_slot_win_stats,
                );
                let b_count = get_participation_count(
                    b_id,
                    slot,
                    bracket_slot_stats,
                    bracket_slot_win_stats,
                );

                // Winner is the team with higher win count at this slot
                let a_wins = bracket_slot_win_stats
                    .get(&a_id)
                    .and_then(|s| s.round_of_32.get(&slot).copied())
                    .unwrap_or(0);
                let b_wins = bracket_slot_win_stats
                    .get(&b_id)
                    .and_then(|s| s.round_of_32.get(&slot).copied())
                    .unwrap_or(0);

                let winner = if a_wins >= b_wins { a_id } else { b_id };

                r32_matches.push(OptimalR32Match {
                    slot,
                    team_a: MostLikelyBracketSlot {
                        team_id: a_id,
                        count: a_count,
                        probability: a_count as f64 / total_simulations as f64,
                    },
                    team_b: MostLikelyBracketSlot {
                        team_id: b_id,
                        count: b_count,
                        probability: b_count as f64 / total_simulations as f64,
                    },
                    winner,
                });
            }
            _ => {
                // Should not happen if we have enough teams
                // Skip this match
            }
        }
    }

    // Propagate winners through later rounds
    let (r16, qf, sf, champion) = propagate_winners(
        &r32_matches,
        bracket_slot_win_stats,
        total_simulations,
    );

    // Compute joint probability (product of all slot probabilities)
    let (joint_prob, log_prob) =
        compute_joint_probability(&r32_matches, &r16, &qf, &sf, &champion);

    OptimalBracket {
        round_of_32: r32_matches,
        round_of_16: r16,
        quarter_finals: qf,
        semi_finals: sf,
        champion,
        joint_probability: joint_prob,
        log_probability: log_prob,
    }
}

/// Propagate winners through R16, QF, SF, and Final.
fn propagate_winners(
    r32_matches: &[OptimalR32Match],
    bracket_slot_win_stats: &HashMap<TeamId, BracketSlotWinStats>,
    total_simulations: u32,
) -> (
    HashMap<u8, MostLikelyBracketSlot>,
    HashMap<u8, MostLikelyBracketSlot>,
    HashMap<u8, MostLikelyBracketSlot>,
    Option<MostLikelyBracketSlot>,
) {
    // R16: slot i receives winner from R32 slots 2i and 2i+1
    let mut r16: HashMap<u8, MostLikelyBracketSlot> = HashMap::new();
    let mut r16_feeders: HashMap<u8, Vec<TeamId>> = HashMap::new();

    for slot in 0..8u8 {
        let feeder1 = (slot * 2) as usize;
        let feeder2 = (slot * 2 + 1) as usize;

        let mut candidates: Vec<TeamId> = Vec::new();
        if feeder1 < r32_matches.len() {
            candidates.push(r32_matches[feeder1].winner);
        }
        if feeder2 < r32_matches.len() {
            candidates.push(r32_matches[feeder2].winner);
        }

        r16_feeders.insert(slot, candidates.clone());

        // Pick the candidate with higher win count at this R16 slot
        let winner = pick_winner_for_slot(&candidates, slot, "round_of_16", bracket_slot_win_stats);

        if let Some(team_id) = winner {
            let count = bracket_slot_win_stats
                .get(&team_id)
                .and_then(|s| s.round_of_16.get(&slot).copied())
                .unwrap_or(0);

            r16.insert(
                slot,
                MostLikelyBracketSlot {
                    team_id,
                    count,
                    probability: count as f64 / total_simulations as f64,
                },
            );
        }
    }

    // QF: slot i receives winner from R16 slots 2i and 2i+1
    let mut qf: HashMap<u8, MostLikelyBracketSlot> = HashMap::new();
    let mut qf_feeders: HashMap<u8, Vec<TeamId>> = HashMap::new();

    for slot in 0..4u8 {
        let feeder1 = slot * 2;
        let feeder2 = slot * 2 + 1;

        let mut candidates: Vec<TeamId> = Vec::new();
        if let Some(data) = r16.get(&feeder1) {
            candidates.push(data.team_id);
        }
        if let Some(data) = r16.get(&feeder2) {
            candidates.push(data.team_id);
        }

        qf_feeders.insert(slot, candidates.clone());

        let winner =
            pick_winner_for_slot(&candidates, slot, "quarter_finals", bracket_slot_win_stats);

        if let Some(team_id) = winner {
            let count = bracket_slot_win_stats
                .get(&team_id)
                .and_then(|s| s.quarter_finals.get(&slot).copied())
                .unwrap_or(0);

            qf.insert(
                slot,
                MostLikelyBracketSlot {
                    team_id,
                    count,
                    probability: count as f64 / total_simulations as f64,
                },
            );
        }
    }

    // SF: slot i receives winner from QF slots 2i and 2i+1
    let mut sf: HashMap<u8, MostLikelyBracketSlot> = HashMap::new();
    let mut sf_feeders: HashMap<u8, Vec<TeamId>> = HashMap::new();

    for slot in 0..2u8 {
        let feeder1 = slot * 2;
        let feeder2 = slot * 2 + 1;

        let mut candidates: Vec<TeamId> = Vec::new();
        if let Some(data) = qf.get(&feeder1) {
            candidates.push(data.team_id);
        }
        if let Some(data) = qf.get(&feeder2) {
            candidates.push(data.team_id);
        }

        sf_feeders.insert(slot, candidates.clone());

        let winner = pick_winner_for_slot(&candidates, slot, "semi_finals", bracket_slot_win_stats);

        if let Some(team_id) = winner {
            let count = bracket_slot_win_stats
                .get(&team_id)
                .and_then(|s| s.semi_finals.get(&slot).copied())
                .unwrap_or(0);

            sf.insert(
                slot,
                MostLikelyBracketSlot {
                    team_id,
                    count,
                    probability: count as f64 / total_simulations as f64,
                },
            );
        }
    }

    // Final: winner from SF slots 0 and 1
    let mut final_candidates: Vec<TeamId> = Vec::new();
    if let Some(data) = sf.get(&0) {
        final_candidates.push(data.team_id);
    }
    if let Some(data) = sf.get(&1) {
        final_candidates.push(data.team_id);
    }

    let champion = pick_winner_for_slot(&final_candidates, 0, "final", bracket_slot_win_stats)
        .map(|team_id| {
            let count = bracket_slot_win_stats
                .get(&team_id)
                .map(|s| s.final_match)
                .unwrap_or(0);

            MostLikelyBracketSlot {
                team_id,
                count,
                probability: count as f64 / total_simulations as f64,
            }
        });

    (r16, qf, sf, champion)
}

/// Pick the winner from candidates based on win stats at a given slot/round.
fn pick_winner_for_slot(
    candidates: &[TeamId],
    slot: u8,
    round: &str,
    bracket_slot_win_stats: &HashMap<TeamId, BracketSlotWinStats>,
) -> Option<TeamId> {
    if candidates.is_empty() {
        return None;
    }

    let get_wins = |team_id: TeamId| -> u32 {
        if let Some(stats) = bracket_slot_win_stats.get(&team_id) {
            match round {
                "round_of_16" => stats.round_of_16.get(&slot).copied().unwrap_or(0),
                "quarter_finals" => stats.quarter_finals.get(&slot).copied().unwrap_or(0),
                "semi_finals" => stats.semi_finals.get(&slot).copied().unwrap_or(0),
                "final" => stats.final_match,
                _ => 0,
            }
        } else {
            0
        }
    };

    candidates
        .iter()
        .max_by_key(|&&team_id| get_wins(team_id))
        .copied()
}

/// Compute the joint probability of the bracket.
fn compute_joint_probability(
    r32_matches: &[OptimalR32Match],
    r16: &HashMap<u8, MostLikelyBracketSlot>,
    qf: &HashMap<u8, MostLikelyBracketSlot>,
    sf: &HashMap<u8, MostLikelyBracketSlot>,
    champion: &Option<MostLikelyBracketSlot>,
) -> (f64, f64) {
    let mut log_sum = 0.0f64;

    // R32 probabilities
    for m in r32_matches {
        let p_a = m.team_a.probability.max(1e-10);
        let p_b = m.team_b.probability.max(1e-10);
        log_sum += p_a.ln() + p_b.ln();
    }

    // R16 probabilities
    for data in r16.values() {
        let p = data.probability.max(1e-10);
        log_sum += p.ln();
    }

    // QF probabilities
    for data in qf.values() {
        let p = data.probability.max(1e-10);
        log_sum += p.ln();
    }

    // SF probabilities
    for data in sf.values() {
        let p = data.probability.max(1e-10);
        log_sum += p.ln();
    }

    // Champion probability
    if let Some(data) = champion {
        let p = data.probability.max(1e-10);
        log_sum += p.ln();
    }

    (log_sum.exp(), log_sum)
}

/// Verify that the optimal bracket has exactly 32 unique teams in R32.
pub fn verify_optimal_bracket(bracket: &OptimalBracket) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    let mut all_teams: HashSet<TeamId> = HashSet::new();

    // Check R32 teams
    for m in &bracket.round_of_32 {
        if all_teams.contains(&m.team_a.team_id) {
            errors.push(format!(
                "Duplicate team in R32: {:?} (slot {} team_a)",
                m.team_a.team_id, m.slot
            ));
        }
        all_teams.insert(m.team_a.team_id);

        if all_teams.contains(&m.team_b.team_id) {
            errors.push(format!(
                "Duplicate team in R32: {:?} (slot {} team_b)",
                m.team_b.team_id, m.slot
            ));
        }
        all_teams.insert(m.team_b.team_id);
    }

    if all_teams.len() != 32 {
        errors.push(format!(
            "Expected 32 unique teams in R32, found {}",
            all_teams.len()
        ));
    }

    // Check R16 winners come from R32 feeders
    for (slot, data) in &bracket.round_of_16 {
        let feeder1 = (*slot * 2) as usize;
        let feeder2 = (*slot * 2 + 1) as usize;

        let valid_feeders: HashSet<TeamId> = bracket
            .round_of_32
            .get(feeder1)
            .map(|m| m.winner)
            .into_iter()
            .chain(
                bracket
                    .round_of_32
                    .get(feeder2)
                    .map(|m| m.winner)
                    .into_iter(),
            )
            .collect();

        if !valid_feeders.contains(&data.team_id) {
            errors.push(format!(
                "R16 slot {} winner {:?} not from feeders {:?}",
                slot, data.team_id, valid_feeders
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wc_core::{Confederation, Group, GroupId, Team, Tournament};

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

    #[test]
    fn test_eligibility_map_built() {
        let tournament = create_test_tournament();
        let team_to_group = build_team_to_group_map(&tournament.groups);
        let team_ids: Vec<TeamId> = tournament.teams.iter().map(|t| t.id).collect();
        let eligibility = build_eligibility_map(&team_ids, &team_to_group);

        // All 48 teams should have some eligibility
        assert_eq!(eligibility.len(), 48);

        // Check that eligibility respects group constraints
        // Team 0 is in Group A (index 0), so can only be at positions for Group A
        let team_0_positions = eligibility.get(&TeamId(0)).unwrap();
        assert!(!team_0_positions.is_empty());

        // Verify Team 0 can be at slot 2 (M73: 2A vs 2B) - position 0 (team_a = 2A)
        // But Team 0 is in Group A, and M73 has 2A vs 2B, so Team 0 can be 2A
        // Slots: check R32_BRACKET for Group A positions
    }

    #[test]
    fn test_position_conversion() {
        assert_eq!(position_to_index(0, 0), 0);
        assert_eq!(position_to_index(0, 1), 1);
        assert_eq!(position_to_index(1, 0), 2);
        assert_eq!(position_to_index(15, 1), 31);

        assert_eq!(index_to_position(0), (0, 0));
        assert_eq!(index_to_position(1), (0, 1));
        assert_eq!(index_to_position(31), (15, 1));
    }

    #[test]
    fn test_optimal_bracket_with_mock_data() {
        let tournament = create_test_tournament();
        let team_ids: Vec<TeamId> = tournament.teams.iter().map(|t| t.id).collect();

        // Create mock stats where each team has some participation in their eligible slots
        let mut bracket_slot_stats: HashMap<TeamId, BracketSlotStats> = HashMap::new();
        let mut bracket_slot_win_stats: HashMap<TeamId, BracketSlotWinStats> = HashMap::new();

        for team in &tournament.teams {
            let mut part_stats = BracketSlotStats::default();
            let mut win_stats = BracketSlotWinStats::default();

            // Give each team some participation in slot 0 (for simplicity)
            part_stats.round_of_32.insert(0, 100);
            win_stats.round_of_32.insert(0, 50);

            bracket_slot_stats.insert(team.id, part_stats);
            bracket_slot_win_stats.insert(team.id, win_stats);
        }

        let bracket = compute_optimal_bracket(
            &team_ids,
            &tournament.groups,
            &bracket_slot_stats,
            &bracket_slot_win_stats,
            1000,
        );

        // Should have some R32 matches
        assert!(!bracket.round_of_32.is_empty());
    }
}
