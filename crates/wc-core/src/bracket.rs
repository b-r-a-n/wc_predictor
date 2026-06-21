//! FIFA World Cup 2026 bracket configuration and pairing logic.
//!
//! This module implements the official FIFA 2026 Round of 32 bracket structure,
//! including the 495-combination third-place team assignment lookup table.

use crate::team::TeamId;
use std::collections::HashMap;

/// Position within a group (1st, 2nd, or 3rd place).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupPosition {
    Winner,
    RunnerUp,
    Third,
}

/// Source of a team for a bracket slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotSource {
    /// A specific group position (e.g., Winner of Group A, Runner-up of Group B).
    GroupTeam {
        group: char,
        position: GroupPosition,
    },
    /// A third-place team from a pool of possible groups.
    /// The actual group is determined by the third-place assignment lookup.
    ThirdPlacePool {
        /// Index into the third-place slot assignments (0-7).
        slot_index: u8,
    },
}

/// A single Round of 32 match pairing.
#[derive(Debug, Clone)]
pub struct R32Match {
    /// FIFA match number (73-88).
    pub match_num: u8,
    /// Source for team A (typically the higher seed).
    pub team_a: SlotSource,
    /// Source for team B.
    pub team_b: SlotSource,
}

/// The official FIFA 2026 Round of 32 bracket structure.
///
/// Matches are ordered so that .chunks(2) pairs correctly feed into Round of 16:
/// - R32[0] winner vs R32[1] winner -> R16[0]
/// - R32[2] winner vs R32[3] winner -> R16[1]
/// - etc.
///
/// FIFA R16 pairings (from regulations):
///   M89: W74 vs W77    M93: W83 vs W84
///   M90: W73 vs W75    M94: W81 vs W82
///   M91: W76 vs W78    M95: W86 vs W88
///   M92: W79 vs W80    M96: W85 vs W87
pub const R32_BRACKET: [R32Match; 16] = [
    // Bracket Half 1 (feeds into QF A and QF C)
    // R16 Match 89: W74 vs W77
    R32Match {
        match_num: 74,
        team_a: SlotSource::GroupTeam { group: 'E', position: GroupPosition::Winner },
        team_b: SlotSource::ThirdPlacePool { slot_index: 0 }, // 3(A/B/C/D/F)
    },
    R32Match {
        match_num: 77,
        team_a: SlotSource::GroupTeam { group: 'I', position: GroupPosition::Winner },
        team_b: SlotSource::ThirdPlacePool { slot_index: 1 }, // 3(C/D/F/G/H)
    },
    // R16 Match 90: W73 vs W75
    R32Match {
        match_num: 73,
        team_a: SlotSource::GroupTeam { group: 'A', position: GroupPosition::RunnerUp },
        team_b: SlotSource::GroupTeam { group: 'B', position: GroupPosition::RunnerUp },
    },
    R32Match {
        match_num: 75,
        team_a: SlotSource::GroupTeam { group: 'F', position: GroupPosition::Winner },
        team_b: SlotSource::GroupTeam { group: 'C', position: GroupPosition::RunnerUp },
    },
    // R16 Match 91: W76 vs W78
    R32Match {
        match_num: 76,
        team_a: SlotSource::GroupTeam { group: 'C', position: GroupPosition::Winner },
        team_b: SlotSource::GroupTeam { group: 'F', position: GroupPosition::RunnerUp },
    },
    R32Match {
        match_num: 78,
        team_a: SlotSource::GroupTeam { group: 'E', position: GroupPosition::RunnerUp },
        team_b: SlotSource::GroupTeam { group: 'I', position: GroupPosition::RunnerUp },
    },
    // R16 Match 92: W79 vs W80
    R32Match {
        match_num: 79,
        team_a: SlotSource::GroupTeam { group: 'A', position: GroupPosition::Winner },
        team_b: SlotSource::ThirdPlacePool { slot_index: 2 }, // 3(C/E/F/H/I)
    },
    R32Match {
        match_num: 80,
        team_a: SlotSource::GroupTeam { group: 'L', position: GroupPosition::Winner },
        team_b: SlotSource::ThirdPlacePool { slot_index: 3 }, // 3(E/H/I/J/K)
    },

    // Bracket Half 2 (feeds into QF B and QF D)
    // R16 Match 93: W83 vs W84
    R32Match {
        match_num: 83,
        team_a: SlotSource::GroupTeam { group: 'K', position: GroupPosition::RunnerUp },
        team_b: SlotSource::GroupTeam { group: 'L', position: GroupPosition::RunnerUp },
    },
    R32Match {
        match_num: 84,
        team_a: SlotSource::GroupTeam { group: 'H', position: GroupPosition::Winner },
        team_b: SlotSource::GroupTeam { group: 'J', position: GroupPosition::RunnerUp },
    },
    // R16 Match 94: W81 vs W82
    R32Match {
        match_num: 81,
        team_a: SlotSource::GroupTeam { group: 'D', position: GroupPosition::Winner },
        team_b: SlotSource::ThirdPlacePool { slot_index: 4 }, // 3(B/E/F/I/J)
    },
    R32Match {
        match_num: 82,
        team_a: SlotSource::GroupTeam { group: 'G', position: GroupPosition::Winner },
        team_b: SlotSource::ThirdPlacePool { slot_index: 5 }, // 3(A/E/H/I/J)
    },
    // R16 Match 95: W86 vs W88
    R32Match {
        match_num: 86,
        team_a: SlotSource::GroupTeam { group: 'J', position: GroupPosition::Winner },
        team_b: SlotSource::GroupTeam { group: 'H', position: GroupPosition::RunnerUp },
    },
    R32Match {
        match_num: 88,
        team_a: SlotSource::GroupTeam { group: 'D', position: GroupPosition::RunnerUp },
        team_b: SlotSource::GroupTeam { group: 'G', position: GroupPosition::RunnerUp },
    },
    // R16 Match 96: W85 vs W87
    R32Match {
        match_num: 85,
        team_a: SlotSource::GroupTeam { group: 'B', position: GroupPosition::Winner },
        team_b: SlotSource::ThirdPlacePool { slot_index: 6 }, // 3(E/F/G/I/J)
    },
    R32Match {
        match_num: 87,
        team_a: SlotSource::GroupTeam { group: 'K', position: GroupPosition::Winner },
        team_b: SlotSource::ThirdPlacePool { slot_index: 7 }, // 3(D/E/I/J/L)
    },
];

/// Mapping of third-place slot indices to the first-place teams they play.
/// Order: 1E, 1I, 1A, 1L, 1D, 1G, 1B, 1K
pub const THIRD_PLACE_OPPONENTS: [char; 8] = ['E', 'I', 'A', 'L', 'D', 'G', 'B', 'K'];

/// Pool of possible source groups for each third-place bracket slot.
/// Index order: [1E, 1I, 1A, 1L, 1D, 1G, 1B, 1K]
const THIRD_PLACE_POOLS: [[char; 5]; 8] = [
    ['A', 'B', 'C', 'D', 'F'],  // 1E plays 3rd from A/B/C/D/F
    ['C', 'D', 'F', 'G', 'H'],  // 1I plays 3rd from C/D/F/G/H
    ['C', 'E', 'F', 'H', 'I'],  // 1A plays 3rd from C/E/F/H/I
    ['E', 'H', 'I', 'J', 'K'],  // 1L plays 3rd from E/H/I/J/K
    ['B', 'E', 'F', 'I', 'J'],  // 1D plays 3rd from B/E/F/I/J
    ['A', 'E', 'H', 'I', 'J'],  // 1G plays 3rd from A/E/H/I/J
    ['E', 'F', 'G', 'I', 'J'],  // 1B plays 3rd from E/F/G/I/J
    ['D', 'E', 'I', 'J', 'L'],  // 1K plays 3rd from D/E/I/J/L
];

/// Third-place assignment lookup table.
///
/// Given the sorted group letters of the 8 qualifying third-place teams,
/// returns the mapping of each first-place team to their third-place opponent.
///
/// Key: 8 uppercase letters (sorted) representing which groups' third-place teams qualify.
/// Value: Array of 8 group letters indicating which third-place team plays each slot.
///        Index corresponds to THIRD_PLACE_OPPONENTS order.
pub fn get_third_place_assignments(qualifying_groups: &str) -> Option<[char; 8]> {
    // First try the official FIFA lookup table
    if let Some(assignments) = THIRD_PLACE_TABLE.get(qualifying_groups) {
        return Some(*assignments);
    }

    // Fallback: compute assignments using pool constraints
    // This ensures each slot gets a valid team from its pool
    compute_third_place_assignments(qualifying_groups)
}

/// Compute third-place assignments when not found in lookup table.
/// Uses a backtracking algorithm that respects pool constraints.
fn compute_third_place_assignments(qualifying_groups: &str) -> Option<[char; 8]> {
    let qualifying: Vec<char> = qualifying_groups.chars().collect();
    if qualifying.len() != 8 {
        return None;
    }

    let mut assignments = [' '; 8];
    let mut used = [false; 8];

    if backtrack_assign(0, &qualifying, &mut assignments, &mut used) {
        Some(assignments)
    } else {
        None
    }
}

/// Backtracking helper for third-place assignment.
fn backtrack_assign(
    slot_idx: usize,
    qualifying: &[char],
    assignments: &mut [char; 8],
    used: &mut [bool; 8],
) -> bool {
    if slot_idx == 8 {
        return true; // All slots assigned successfully
    }

    let pool = &THIRD_PLACE_POOLS[slot_idx];

    // Try each group in the pool
    for &pool_group in pool.iter() {
        // Check if this group is qualifying and not yet used
        if let Some(qual_idx) = qualifying.iter().position(|&g| g == pool_group) {
            if !used[qual_idx] {
                // Try this assignment
                assignments[slot_idx] = pool_group;
                used[qual_idx] = true;

                // Recurse to next slot
                if backtrack_assign(slot_idx + 1, qualifying, assignments, used) {
                    return true;
                }

                // Backtrack
                used[qual_idx] = false;
            }
        }
    }

    false // No valid assignment found for this slot
}

/// Resolve R32 pairings given group results.
///
/// Returns 16 pairs of TeamIds in bracket order.
pub fn build_r32_pairings(
    winners: &HashMap<char, TeamId>,
    runners_up: &HashMap<char, TeamId>,
    qualifying_thirds: &[(char, TeamId)], // Sorted by rank, contains group letter and team
) -> Vec<(TeamId, TeamId)> {
    // Build the qualifying groups key (sorted group letters)
    let mut group_letters: Vec<char> = qualifying_thirds.iter().map(|(g, _)| *g).collect();
    group_letters.sort();
    let key: String = group_letters.iter().collect();

    // Get third-place assignments for this combination
    let third_assignments = get_third_place_assignments(&key)
        .unwrap_or_else(|| {
            // This should never happen as compute_third_place_assignments handles all valid cases
            panic!("Cannot assign third-place teams for combination: {} - check pool constraints", key)
        });

    // Build a map from group letter to third-place team
    let third_place_map: HashMap<char, TeamId> = qualifying_thirds.iter().cloned().collect();

    // Build the 16 R32 pairings
    R32_BRACKET
        .iter()
        .map(|m| {
            let team_a = resolve_slot(&m.team_a, winners, runners_up, &third_assignments, &third_place_map);
            let team_b = resolve_slot(&m.team_b, winners, runners_up, &third_assignments, &third_place_map);
            (team_a, team_b)
        })
        .collect()
}

fn resolve_slot(
    source: &SlotSource,
    winners: &HashMap<char, TeamId>,
    runners_up: &HashMap<char, TeamId>,
    third_assignments: &[char; 8],
    third_place_map: &HashMap<char, TeamId>,
) -> TeamId {
    match source {
        SlotSource::GroupTeam { group, position } => {
            match position {
                GroupPosition::Winner => *winners.get(group).expect("Missing group winner"),
                GroupPosition::RunnerUp => *runners_up.get(group).expect("Missing runner-up"),
                GroupPosition::Third => *third_place_map.get(group).expect("Missing third place"),
            }
        }
        SlotSource::ThirdPlacePool { slot_index } => {
            let group = third_assignments[*slot_index as usize];
            *third_place_map.get(&group).expect("Missing third-place team for slot")
        }
    }
}

// =============================================================================
// Third-Place Assignment Lookup Table (495 combinations)
// =============================================================================
//
// Official FIFA 2026 Annex C mapping: each combination of 8 qualifying
// third-place groups (chosen from the 12 groups A-L) -> the third-place group
// assigned to each winner's R32 slot.
//
// Key:   Sorted 8-letter string of qualifying groups (e.g., "ABCDEFGH").
// Value: [3rd for 1E, 3rd for 1I, 3rd for 1A, 3rd for 1L,
//         3rd for 1D, 3rd for 1G, 3rd for 1B, 3rd for 1K]
//        (THIRD_PLACE_OPPONENTS order; R32 matches 74,77,79,80,81,82,85,87).
//
// Verified against two independent encodings of Annex C (the FIFA Competition
// Regulations and the Wikipedia Annex C template), which agree on all 495 rows.
// Every row is bijective and respects the THIRD_PLACE_POOLS constraints and the
// no-same-group rule -- see tests::test_third_place_table_is_complete_and_valid,
// which would have caught the previously-cleared bad data.

use std::sync::LazyLock;

const THIRD_PLACE_DATA: [(&str, [char; 8]); 495] = [
    ("ABCDEFGH", ['C','F','H','E','B','A','G','D']),
    ("ABCDEFGI", ['D','F','C','I','B','A','G','E']),
    ("ABCDEFGJ", ['D','F','C','J','B','A','G','E']),
    ("ABCDEFGK", ['D','F','C','K','B','A','G','E']),
    ("ABCDEFGL", ['D','F','C','E','B','A','G','L']),
    ("ABCDEFHI", ['C','F','H','I','B','A','E','D']),
    ("ABCDEFHJ", ['C','F','H','E','B','A','J','D']),
    ("ABCDEFHK", ['C','F','H','K','B','A','E','D']),
    ("ABCDEFHL", ['C','D','H','E','B','A','F','L']),
    ("ABCDEFIJ", ['D','F','C','I','B','A','J','E']),
    ("ABCDEFIK", ['D','F','C','K','B','A','E','I']),
    ("ABCDEFIL", ['D','F','C','I','B','A','E','L']),
    ("ABCDEFJK", ['D','F','C','K','B','A','J','E']),
    ("ABCDEFJL", ['D','F','C','E','B','A','J','L']),
    ("ABCDEFKL", ['D','F','C','K','B','A','E','L']),
    ("ABCDEGHI", ['C','D','H','I','B','A','G','E']),
    ("ABCDEGHJ", ['C','D','H','J','B','A','G','E']),
    ("ABCDEGHK", ['C','D','H','K','B','A','G','E']),
    ("ABCDEGHL", ['C','D','H','E','B','A','G','L']),
    ("ABCDEGIJ", ['C','D','E','J','B','A','G','I']),
    ("ABCDEGIK", ['C','D','E','K','B','A','G','I']),
    ("ABCDEGIL", ['C','D','E','I','B','A','G','L']),
    ("ABCDEGJK", ['C','D','E','K','B','A','G','J']),
    ("ABCDEGJL", ['C','D','E','J','B','A','G','L']),
    ("ABCDEGKL", ['C','D','E','K','B','A','G','L']),
    ("ABCDEHIJ", ['C','D','H','I','B','A','J','E']),
    ("ABCDEHIK", ['C','D','H','K','B','A','E','I']),
    ("ABCDEHIL", ['C','D','H','I','B','A','E','L']),
    ("ABCDEHJK", ['C','D','H','K','B','A','J','E']),
    ("ABCDEHJL", ['C','D','H','E','B','A','J','L']),
    ("ABCDEHKL", ['C','D','H','K','B','A','E','L']),
    ("ABCDEIJK", ['C','D','E','K','B','A','J','I']),
    ("ABCDEIJL", ['C','D','E','I','B','A','J','L']),
    ("ABCDEIKL", ['C','D','E','K','B','A','I','L']),
    ("ABCDEJKL", ['C','D','E','K','B','A','J','L']),
    ("ABCDFGHI", ['C','F','H','I','B','A','G','D']),
    ("ABCDFGHJ", ['C','F','H','J','B','A','G','D']),
    ("ABCDFGHK", ['C','F','H','K','B','A','G','D']),
    ("ABCDFGHL", ['D','F','C','H','B','A','G','L']),
    ("ABCDFGIJ", ['D','F','C','J','B','A','G','I']),
    ("ABCDFGIK", ['D','F','C','K','B','A','G','I']),
    ("ABCDFGIL", ['D','F','C','I','B','A','G','L']),
    ("ABCDFGJK", ['D','F','C','K','B','A','G','J']),
    ("ABCDFGJL", ['D','F','C','J','B','A','G','L']),
    ("ABCDFGKL", ['D','F','C','K','B','A','G','L']),
    ("ABCDFHIJ", ['C','F','H','I','B','A','J','D']),
    ("ABCDFHIK", ['C','D','H','K','B','A','F','I']),
    ("ABCDFHIL", ['C','D','H','I','B','A','F','L']),
    ("ABCDFHJK", ['C','F','H','K','B','A','J','D']),
    ("ABCDFHJL", ['D','F','C','H','B','A','J','L']),
    ("ABCDFHKL", ['C','D','H','K','B','A','F','L']),
    ("ABCDFIJK", ['D','F','C','K','B','A','J','I']),
    ("ABCDFIJL", ['D','F','C','I','B','A','J','L']),
    ("ABCDFIKL", ['D','F','C','K','B','A','I','L']),
    ("ABCDFJKL", ['D','F','C','K','B','A','J','L']),
    ("ABCDGHIJ", ['C','D','H','J','B','A','G','I']),
    ("ABCDGHIK", ['C','D','H','K','B','A','G','I']),
    ("ABCDGHIL", ['C','D','H','I','B','A','G','L']),
    ("ABCDGHJK", ['C','D','H','K','B','A','G','J']),
    ("ABCDGHJL", ['C','D','H','J','B','A','G','L']),
    ("ABCDGHKL", ['C','D','H','K','B','A','G','L']),
    ("ABCDGIJK", ['D','G','C','K','B','A','J','I']),
    ("ABCDGIJL", ['D','G','C','I','B','A','J','L']),
    ("ABCDGIKL", ['C','D','I','K','B','A','G','L']),
    ("ABCDGJKL", ['D','G','C','K','B','A','J','L']),
    ("ABCDHIJK", ['C','D','H','K','B','A','J','I']),
    ("ABCDHIJL", ['C','D','H','I','B','A','J','L']),
    ("ABCDHIKL", ['C','D','H','K','B','A','I','L']),
    ("ABCDHJKL", ['C','D','H','K','B','A','J','L']),
    ("ABCDIJKL", ['C','D','I','K','B','A','J','L']),
    ("ABCEFGHI", ['C','F','H','I','B','A','G','E']),
    ("ABCEFGHJ", ['C','F','H','J','B','A','G','E']),
    ("ABCEFGHK", ['C','F','H','K','B','A','G','E']),
    ("ABCEFGHL", ['C','F','H','E','B','A','G','L']),
    ("ABCEFGIJ", ['C','F','E','J','B','A','G','I']),
    ("ABCEFGIK", ['C','F','E','K','B','A','G','I']),
    ("ABCEFGIL", ['C','F','E','I','B','A','G','L']),
    ("ABCEFGJK", ['C','F','E','K','B','A','G','J']),
    ("ABCEFGJL", ['C','F','E','J','B','A','G','L']),
    ("ABCEFGKL", ['C','F','E','K','B','A','G','L']),
    ("ABCEFHIJ", ['C','F','H','I','B','A','J','E']),
    ("ABCEFHIK", ['C','F','H','K','B','A','E','I']),
    ("ABCEFHIL", ['C','F','H','I','B','A','E','L']),
    ("ABCEFHJK", ['C','F','H','K','B','A','J','E']),
    ("ABCEFHJL", ['C','F','H','E','B','A','J','L']),
    ("ABCEFHKL", ['C','F','H','K','B','A','E','L']),
    ("ABCEFIJK", ['C','F','E','K','B','A','J','I']),
    ("ABCEFIJL", ['C','F','E','I','B','A','J','L']),
    ("ABCEFIKL", ['C','F','E','K','B','A','I','L']),
    ("ABCEFJKL", ['C','F','E','K','B','A','J','L']),
    ("ABCEGHIJ", ['C','G','H','I','B','A','J','E']),
    ("ABCEGHIK", ['C','H','E','K','B','A','G','I']),
    ("ABCEGHIL", ['C','H','E','I','B','A','G','L']),
    ("ABCEGHJK", ['C','G','H','K','B','A','J','E']),
    ("ABCEGHJL", ['C','G','H','E','B','A','J','L']),
    ("ABCEGHKL", ['C','H','E','K','B','A','G','L']),
    ("ABCEGIJK", ['C','G','E','K','B','A','J','I']),
    ("ABCEGIJL", ['C','G','E','I','B','A','J','L']),
    ("ABCEGIKL", ['A','C','E','K','B','I','G','L']),
    ("ABCEGJKL", ['C','G','E','K','B','A','J','L']),
    ("ABCEHIJK", ['C','H','E','K','B','A','J','I']),
    ("ABCEHIJL", ['C','H','E','I','B','A','J','L']),
    ("ABCEHIKL", ['C','H','E','K','B','A','I','L']),
    ("ABCEHJKL", ['C','H','E','K','B','A','J','L']),
    ("ABCEIJKL", ['A','C','E','K','B','I','J','L']),
    ("ABCFGHIJ", ['C','F','H','J','B','A','G','I']),
    ("ABCFGHIK", ['C','F','H','K','B','A','G','I']),
    ("ABCFGHIL", ['C','F','H','I','B','A','G','L']),
    ("ABCFGHJK", ['C','F','H','K','B','A','G','J']),
    ("ABCFGHJL", ['C','F','H','J','B','A','G','L']),
    ("ABCFGHKL", ['C','F','H','K','B','A','G','L']),
    ("ABCFGIJK", ['F','G','C','K','B','A','J','I']),
    ("ABCFGIJL", ['F','G','C','I','B','A','J','L']),
    ("ABCFGIKL", ['C','F','I','K','B','A','G','L']),
    ("ABCFGJKL", ['F','G','C','K','B','A','J','L']),
    ("ABCFHIJK", ['C','F','H','K','B','A','J','I']),
    ("ABCFHIJL", ['C','F','H','I','B','A','J','L']),
    ("ABCFHIKL", ['C','F','H','K','B','A','I','L']),
    ("ABCFHJKL", ['C','F','H','K','B','A','J','L']),
    ("ABCFIJKL", ['C','F','I','K','B','A','J','L']),
    ("ABCGHIJK", ['C','G','H','K','B','A','J','I']),
    ("ABCGHIJL", ['C','G','H','I','B','A','J','L']),
    ("ABCGHIKL", ['C','H','I','K','B','A','G','L']),
    ("ABCGHJKL", ['C','G','H','K','B','A','J','L']),
    ("ABCGIJKL", ['C','G','I','K','B','A','J','L']),
    ("ABCHIJKL", ['C','H','I','K','B','A','J','L']),
    ("ABDEFGHI", ['D','F','H','I','B','A','G','E']),
    ("ABDEFGHJ", ['D','F','H','J','B','A','G','E']),
    ("ABDEFGHK", ['D','F','H','K','B','A','G','E']),
    ("ABDEFGHL", ['D','F','H','E','B','A','G','L']),
    ("ABDEFGIJ", ['D','F','E','J','B','A','G','I']),
    ("ABDEFGIK", ['D','F','E','K','B','A','G','I']),
    ("ABDEFGIL", ['D','F','E','I','B','A','G','L']),
    ("ABDEFGJK", ['D','F','E','K','B','A','G','J']),
    ("ABDEFGJL", ['D','F','E','J','B','A','G','L']),
    ("ABDEFGKL", ['D','F','E','K','B','A','G','L']),
    ("ABDEFHIJ", ['D','F','H','I','B','A','J','E']),
    ("ABDEFHIK", ['D','F','H','K','B','A','E','I']),
    ("ABDEFHIL", ['D','F','H','I','B','A','E','L']),
    ("ABDEFHJK", ['D','F','H','K','B','A','J','E']),
    ("ABDEFHJL", ['D','F','H','E','B','A','J','L']),
    ("ABDEFHKL", ['D','F','H','K','B','A','E','L']),
    ("ABDEFIJK", ['D','F','E','K','B','A','J','I']),
    ("ABDEFIJL", ['D','F','E','I','B','A','J','L']),
    ("ABDEFIKL", ['D','F','E','K','B','A','I','L']),
    ("ABDEFJKL", ['D','F','E','K','B','A','J','L']),
    ("ABDEGHIJ", ['D','G','H','I','B','A','J','E']),
    ("ABDEGHIK", ['D','H','E','K','B','A','G','I']),
    ("ABDEGHIL", ['D','H','E','I','B','A','G','L']),
    ("ABDEGHJK", ['D','G','H','K','B','A','J','E']),
    ("ABDEGHJL", ['D','G','H','E','B','A','J','L']),
    ("ABDEGHKL", ['D','H','E','K','B','A','G','L']),
    ("ABDEGIJK", ['D','G','E','K','B','A','J','I']),
    ("ABDEGIJL", ['D','G','E','I','B','A','J','L']),
    ("ABDEGIKL", ['A','D','E','K','B','I','G','L']),
    ("ABDEGJKL", ['D','G','E','K','B','A','J','L']),
    ("ABDEHIJK", ['D','H','E','K','B','A','J','I']),
    ("ABDEHIJL", ['D','H','E','I','B','A','J','L']),
    ("ABDEHIKL", ['D','H','E','K','B','A','I','L']),
    ("ABDEHJKL", ['D','H','E','K','B','A','J','L']),
    ("ABDEIJKL", ['A','D','E','K','B','I','J','L']),
    ("ABDFGHIJ", ['D','F','H','J','B','A','G','I']),
    ("ABDFGHIK", ['D','F','H','K','B','A','G','I']),
    ("ABDFGHIL", ['D','F','H','I','B','A','G','L']),
    ("ABDFGHJK", ['D','F','H','K','B','A','G','J']),
    ("ABDFGHJL", ['D','F','H','J','B','A','G','L']),
    ("ABDFGHKL", ['D','F','H','K','B','A','G','L']),
    ("ABDFGIJK", ['D','G','F','K','B','A','J','I']),
    ("ABDFGIJL", ['D','G','F','I','B','A','J','L']),
    ("ABDFGIKL", ['D','F','I','K','B','A','G','L']),
    ("ABDFGJKL", ['D','G','F','K','B','A','J','L']),
    ("ABDFHIJK", ['D','F','H','K','B','A','J','I']),
    ("ABDFHIJL", ['D','F','H','I','B','A','J','L']),
    ("ABDFHIKL", ['D','F','H','K','B','A','I','L']),
    ("ABDFHJKL", ['D','F','H','K','B','A','J','L']),
    ("ABDFIJKL", ['D','F','I','K','B','A','J','L']),
    ("ABDGHIJK", ['D','G','H','K','B','A','J','I']),
    ("ABDGHIJL", ['D','G','H','I','B','A','J','L']),
    ("ABDGHIKL", ['D','H','I','K','B','A','G','L']),
    ("ABDGHJKL", ['D','G','H','K','B','A','J','L']),
    ("ABDGIJKL", ['D','G','I','K','B','A','J','L']),
    ("ABDHIJKL", ['D','H','I','K','B','A','J','L']),
    ("ABEFGHIJ", ['F','G','H','I','B','A','J','E']),
    ("ABEFGHIK", ['F','H','E','K','B','A','G','I']),
    ("ABEFGHIL", ['F','H','E','I','B','A','G','L']),
    ("ABEFGHJK", ['F','G','H','K','B','A','J','E']),
    ("ABEFGHJL", ['F','G','H','E','B','A','J','L']),
    ("ABEFGHKL", ['F','H','E','K','B','A','G','L']),
    ("ABEFGIJK", ['F','G','E','K','B','A','J','I']),
    ("ABEFGIJL", ['F','G','E','I','B','A','J','L']),
    ("ABEFGIKL", ['A','F','E','K','B','I','G','L']),
    ("ABEFGJKL", ['F','G','E','K','B','A','J','L']),
    ("ABEFHIJK", ['F','H','E','K','B','A','J','I']),
    ("ABEFHIJL", ['F','H','E','I','B','A','J','L']),
    ("ABEFHIKL", ['F','H','E','K','B','A','I','L']),
    ("ABEFHJKL", ['F','H','E','K','B','A','J','L']),
    ("ABEFIJKL", ['A','F','E','K','B','I','J','L']),
    ("ABEGHIJK", ['A','G','E','K','B','H','J','I']),
    ("ABEGHIJL", ['A','G','E','I','B','H','J','L']),
    ("ABEGHIKL", ['A','H','E','K','B','I','G','L']),
    ("ABEGHJKL", ['A','G','E','K','B','H','J','L']),
    ("ABEGIJKL", ['A','G','E','K','B','I','J','L']),
    ("ABEHIJKL", ['A','H','E','K','B','I','J','L']),
    ("ABFGHIJK", ['F','G','H','K','B','A','J','I']),
    ("ABFGHIJL", ['F','G','H','I','B','A','J','L']),
    ("ABFGHIKL", ['A','F','H','K','B','I','G','L']),
    ("ABFGHJKL", ['F','G','H','K','B','A','J','L']),
    ("ABFGIJKL", ['F','G','I','K','B','A','J','L']),
    ("ABFHIJKL", ['A','F','H','K','B','I','J','L']),
    ("ABGHIJKL", ['A','G','H','K','B','I','J','L']),
    ("ACDEFGHI", ['C','F','H','I','E','A','G','D']),
    ("ACDEFGHJ", ['C','F','H','E','J','A','G','D']),
    ("ACDEFGHK", ['C','F','H','K','E','A','G','D']),
    ("ACDEFGHL", ['C','D','H','E','F','A','G','L']),
    ("ACDEFGIJ", ['D','F','C','I','J','A','G','E']),
    ("ACDEFGIK", ['D','F','C','K','E','A','G','I']),
    ("ACDEFGIL", ['D','F','C','I','E','A','G','L']),
    ("ACDEFGJK", ['D','F','C','K','J','A','G','E']),
    ("ACDEFGJL", ['D','F','C','E','J','A','G','L']),
    ("ACDEFGKL", ['D','F','C','K','E','A','G','L']),
    ("ACDEFHIJ", ['C','F','H','I','E','A','J','D']),
    ("ACDEFHIK", ['C','D','H','K','F','A','E','I']),
    ("ACDEFHIL", ['C','D','H','I','F','A','E','L']),
    ("ACDEFHJK", ['C','F','H','K','E','A','J','D']),
    ("ACDEFHJL", ['C','D','H','E','F','A','J','L']),
    ("ACDEFHKL", ['C','D','H','K','F','A','E','L']),
    ("ACDEFIJK", ['D','F','C','K','E','A','J','I']),
    ("ACDEFIJL", ['D','F','C','I','E','A','J','L']),
    ("ACDEFIKL", ['D','F','C','K','I','A','E','L']),
    ("ACDEFJKL", ['D','F','C','K','E','A','J','L']),
    ("ACDEGHIJ", ['C','D','H','I','J','A','G','E']),
    ("ACDEGHIK", ['C','D','H','K','E','A','G','I']),
    ("ACDEGHIL", ['C','D','H','I','E','A','G','L']),
    ("ACDEGHJK", ['C','D','H','K','J','A','G','E']),
    ("ACDEGHJL", ['C','D','H','E','J','A','G','L']),
    ("ACDEGHKL", ['C','D','H','K','E','A','G','L']),
    ("ACDEGIJK", ['C','D','E','K','J','A','G','I']),
    ("ACDEGIJL", ['C','D','E','I','J','A','G','L']),
    ("ACDEGIKL", ['C','D','E','K','I','A','G','L']),
    ("ACDEGJKL", ['C','D','E','K','J','A','G','L']),
    ("ACDEHIJK", ['C','D','H','K','E','A','J','I']),
    ("ACDEHIJL", ['C','D','H','I','E','A','J','L']),
    ("ACDEHIKL", ['C','D','H','K','I','A','E','L']),
    ("ACDEHJKL", ['C','D','H','K','E','A','J','L']),
    ("ACDEIJKL", ['C','D','E','K','I','A','J','L']),
    ("ACDFGHIJ", ['C','F','H','I','J','A','G','D']),
    ("ACDFGHIK", ['C','D','H','K','F','A','G','I']),
    ("ACDFGHIL", ['C','D','H','I','F','A','G','L']),
    ("ACDFGHJK", ['C','F','H','K','J','A','G','D']),
    ("ACDFGHJL", ['D','F','C','H','J','A','G','L']),
    ("ACDFGHKL", ['C','D','H','K','F','A','G','L']),
    ("ACDFGIJK", ['D','F','C','K','J','A','G','I']),
    ("ACDFGIJL", ['D','F','C','I','J','A','G','L']),
    ("ACDFGIKL", ['D','F','C','K','I','A','G','L']),
    ("ACDFGJKL", ['D','F','C','K','J','A','G','L']),
    ("ACDFHIJK", ['C','D','H','K','F','A','J','I']),
    ("ACDFHIJL", ['C','D','H','I','F','A','J','L']),
    ("ACDFHIKL", ['C','D','H','K','I','A','F','L']),
    ("ACDFHJKL", ['C','D','H','K','F','A','J','L']),
    ("ACDFIJKL", ['D','F','C','K','I','A','J','L']),
    ("ACDGHIJK", ['C','D','H','K','J','A','G','I']),
    ("ACDGHIJL", ['C','D','H','I','J','A','G','L']),
    ("ACDGHIKL", ['C','D','H','K','I','A','G','L']),
    ("ACDGHJKL", ['C','D','H','K','J','A','G','L']),
    ("ACDGIJKL", ['C','D','I','K','J','A','G','L']),
    ("ACDHIJKL", ['C','D','H','K','I','A','J','L']),
    ("ACEFGHIJ", ['C','F','H','I','J','A','G','E']),
    ("ACEFGHIK", ['C','F','H','K','E','A','G','I']),
    ("ACEFGHIL", ['C','F','H','I','E','A','G','L']),
    ("ACEFGHJK", ['C','F','H','K','J','A','G','E']),
    ("ACEFGHJL", ['C','F','H','E','J','A','G','L']),
    ("ACEFGHKL", ['C','F','H','K','E','A','G','L']),
    ("ACEFGIJK", ['C','F','E','K','J','A','G','I']),
    ("ACEFGIJL", ['C','F','E','I','J','A','G','L']),
    ("ACEFGIKL", ['C','F','E','K','I','A','G','L']),
    ("ACEFGJKL", ['C','F','E','K','J','A','G','L']),
    ("ACEFHIJK", ['C','F','H','K','E','A','J','I']),
    ("ACEFHIJL", ['C','F','H','I','E','A','J','L']),
    ("ACEFHIKL", ['C','F','H','K','I','A','E','L']),
    ("ACEFHJKL", ['C','F','H','K','E','A','J','L']),
    ("ACEFIJKL", ['C','F','E','K','I','A','J','L']),
    ("ACEGHIJK", ['C','H','E','K','J','A','G','I']),
    ("ACEGHIJL", ['C','H','E','I','J','A','G','L']),
    ("ACEGHIKL", ['C','H','E','K','I','A','G','L']),
    ("ACEGHJKL", ['C','H','E','K','J','A','G','L']),
    ("ACEGIJKL", ['C','G','E','K','I','A','J','L']),
    ("ACEHIJKL", ['C','H','E','K','I','A','J','L']),
    ("ACFGHIJK", ['C','F','H','K','J','A','G','I']),
    ("ACFGHIJL", ['C','F','H','I','J','A','G','L']),
    ("ACFGHIKL", ['C','F','H','K','I','A','G','L']),
    ("ACFGHJKL", ['C','F','H','K','J','A','G','L']),
    ("ACFGIJKL", ['C','F','I','K','J','A','G','L']),
    ("ACFHIJKL", ['C','F','H','K','I','A','J','L']),
    ("ACGHIJKL", ['C','G','H','K','I','A','J','L']),
    ("ADEFGHIJ", ['D','F','H','I','J','A','G','E']),
    ("ADEFGHIK", ['D','F','H','K','E','A','G','I']),
    ("ADEFGHIL", ['D','F','H','I','E','A','G','L']),
    ("ADEFGHJK", ['D','F','H','K','J','A','G','E']),
    ("ADEFGHJL", ['D','F','H','E','J','A','G','L']),
    ("ADEFGHKL", ['D','F','H','K','E','A','G','L']),
    ("ADEFGIJK", ['D','F','E','K','J','A','G','I']),
    ("ADEFGIJL", ['D','F','E','I','J','A','G','L']),
    ("ADEFGIKL", ['D','F','E','K','I','A','G','L']),
    ("ADEFGJKL", ['D','F','E','K','J','A','G','L']),
    ("ADEFHIJK", ['D','F','H','K','E','A','J','I']),
    ("ADEFHIJL", ['D','F','H','I','E','A','J','L']),
    ("ADEFHIKL", ['D','F','H','K','I','A','E','L']),
    ("ADEFHJKL", ['D','F','H','K','E','A','J','L']),
    ("ADEFIJKL", ['D','F','E','K','I','A','J','L']),
    ("ADEGHIJK", ['D','H','E','K','J','A','G','I']),
    ("ADEGHIJL", ['D','H','E','I','J','A','G','L']),
    ("ADEGHIKL", ['D','H','E','K','I','A','G','L']),
    ("ADEGHJKL", ['D','H','E','K','J','A','G','L']),
    ("ADEGIJKL", ['D','G','E','K','I','A','J','L']),
    ("ADEHIJKL", ['D','H','E','K','I','A','J','L']),
    ("ADFGHIJK", ['D','F','H','K','J','A','G','I']),
    ("ADFGHIJL", ['D','F','H','I','J','A','G','L']),
    ("ADFGHIKL", ['D','F','H','K','I','A','G','L']),
    ("ADFGHJKL", ['D','F','H','K','J','A','G','L']),
    ("ADFGIJKL", ['D','F','I','K','J','A','G','L']),
    ("ADFHIJKL", ['D','F','H','K','I','A','J','L']),
    ("ADGHIJKL", ['D','G','H','K','I','A','J','L']),
    ("AEFGHIJK", ['F','H','E','K','J','A','G','I']),
    ("AEFGHIJL", ['F','H','E','I','J','A','G','L']),
    ("AEFGHIKL", ['F','H','E','K','I','A','G','L']),
    ("AEFGHJKL", ['F','H','E','K','J','A','G','L']),
    ("AEFGIJKL", ['F','G','E','K','I','A','J','L']),
    ("AEFHIJKL", ['F','H','E','K','I','A','J','L']),
    ("AEGHIJKL", ['A','G','E','K','I','H','J','L']),
    ("AFGHIJKL", ['F','G','H','K','I','A','J','L']),
    ("BCDEFGHI", ['D','F','C','I','B','H','G','E']),
    ("BCDEFGHJ", ['C','F','H','E','B','J','G','D']),
    ("BCDEFGHK", ['D','F','C','K','B','H','G','E']),
    ("BCDEFGHL", ['D','F','C','E','B','H','G','L']),
    ("BCDEFGIJ", ['D','F','C','I','B','J','G','E']),
    ("BCDEFGIK", ['D','F','C','K','B','E','G','I']),
    ("BCDEFGIL", ['D','F','C','I','B','E','G','L']),
    ("BCDEFGJK", ['D','F','C','K','B','J','G','E']),
    ("BCDEFGJL", ['D','F','C','E','B','J','G','L']),
    ("BCDEFGKL", ['D','F','C','K','B','E','G','L']),
    ("BCDEFHIJ", ['D','F','C','I','B','H','J','E']),
    ("BCDEFHIK", ['D','F','C','K','B','H','E','I']),
    ("BCDEFHIL", ['D','F','C','I','B','H','E','L']),
    ("BCDEFHJK", ['D','F','C','K','B','H','J','E']),
    ("BCDEFHJL", ['D','F','C','E','B','H','J','L']),
    ("BCDEFHKL", ['D','F','C','K','B','H','E','L']),
    ("BCDEFIJK", ['D','F','C','K','B','E','J','I']),
    ("BCDEFIJL", ['D','F','C','I','B','E','J','L']),
    ("BCDEFIKL", ['D','F','C','K','B','I','E','L']),
    ("BCDEFJKL", ['D','F','C','K','B','E','J','L']),
    ("BCDEGHIJ", ['C','D','H','I','B','J','G','E']),
    ("BCDEGHIK", ['C','D','E','K','B','H','G','I']),
    ("BCDEGHIL", ['C','D','E','I','B','H','G','L']),
    ("BCDEGHJK", ['C','D','H','K','B','J','G','E']),
    ("BCDEGHJL", ['C','D','H','E','B','J','G','L']),
    ("BCDEGHKL", ['C','D','E','K','B','H','G','L']),
    ("BCDEGIJK", ['C','D','E','K','B','J','G','I']),
    ("BCDEGIJL", ['C','D','E','I','B','J','G','L']),
    ("BCDEGIKL", ['C','D','E','K','B','I','G','L']),
    ("BCDEGJKL", ['C','D','E','K','B','J','G','L']),
    ("BCDEHIJK", ['C','D','E','K','B','H','J','I']),
    ("BCDEHIJL", ['C','D','E','I','B','H','J','L']),
    ("BCDEHIKL", ['C','D','E','K','B','H','I','L']),
    ("BCDEHJKL", ['C','D','E','K','B','H','J','L']),
    ("BCDEIJKL", ['C','D','E','K','B','I','J','L']),
    ("BCDFGHIJ", ['C','F','H','I','B','J','G','D']),
    ("BCDFGHIK", ['D','F','C','K','B','H','G','I']),
    ("BCDFGHIL", ['D','F','C','I','B','H','G','L']),
    ("BCDFGHJK", ['C','F','H','K','B','J','G','D']),
    ("BCDFGHJL", ['D','F','C','J','B','H','G','L']),
    ("BCDFGHKL", ['D','F','C','K','B','H','G','L']),
    ("BCDFGIJK", ['D','F','C','K','B','J','G','I']),
    ("BCDFGIJL", ['D','F','C','I','B','J','G','L']),
    ("BCDFGIKL", ['D','F','C','K','B','I','G','L']),
    ("BCDFGJKL", ['D','F','C','K','B','J','G','L']),
    ("BCDFHIJK", ['D','F','C','K','B','H','J','I']),
    ("BCDFHIJL", ['D','F','C','I','B','H','J','L']),
    ("BCDFHIKL", ['D','F','C','K','B','H','I','L']),
    ("BCDFHJKL", ['D','F','C','K','B','H','J','L']),
    ("BCDFIJKL", ['D','F','C','K','B','I','J','L']),
    ("BCDGHIJK", ['C','D','H','K','B','J','G','I']),
    ("BCDGHIJL", ['C','D','H','I','B','J','G','L']),
    ("BCDGHIKL", ['C','D','H','K','B','I','G','L']),
    ("BCDGHJKL", ['C','D','H','K','B','J','G','L']),
    ("BCDGIJKL", ['C','D','I','K','B','J','G','L']),
    ("BCDHIJKL", ['C','D','H','K','B','I','J','L']),
    ("BCEFGHIJ", ['C','F','H','I','B','J','G','E']),
    ("BCEFGHIK", ['C','F','E','K','B','H','G','I']),
    ("BCEFGHIL", ['C','F','E','I','B','H','G','L']),
    ("BCEFGHJK", ['C','F','H','K','B','J','G','E']),
    ("BCEFGHJL", ['C','F','H','E','B','J','G','L']),
    ("BCEFGHKL", ['C','F','E','K','B','H','G','L']),
    ("BCEFGIJK", ['C','F','E','K','B','J','G','I']),
    ("BCEFGIJL", ['C','F','E','I','B','J','G','L']),
    ("BCEFGIKL", ['C','F','E','K','B','I','G','L']),
    ("BCEFGJKL", ['C','F','E','K','B','J','G','L']),
    ("BCEFHIJK", ['C','F','E','K','B','H','J','I']),
    ("BCEFHIJL", ['C','F','E','I','B','H','J','L']),
    ("BCEFHIKL", ['C','F','E','K','B','H','I','L']),
    ("BCEFHJKL", ['C','F','E','K','B','H','J','L']),
    ("BCEFIJKL", ['C','F','E','K','B','I','J','L']),
    ("BCEGHIJK", ['C','G','E','K','B','H','J','I']),
    ("BCEGHIJL", ['C','G','E','I','B','H','J','L']),
    ("BCEGHIKL", ['C','H','E','K','B','I','G','L']),
    ("BCEGHJKL", ['C','G','E','K','B','H','J','L']),
    ("BCEGIJKL", ['C','G','E','K','B','I','J','L']),
    ("BCEHIJKL", ['C','H','E','K','B','I','J','L']),
    ("BCFGHIJK", ['C','F','H','K','B','J','G','I']),
    ("BCFGHIJL", ['C','F','H','I','B','J','G','L']),
    ("BCFGHIKL", ['C','F','H','K','B','I','G','L']),
    ("BCFGHJKL", ['C','F','H','K','B','J','G','L']),
    ("BCFGIJKL", ['C','F','I','K','B','J','G','L']),
    ("BCFHIJKL", ['C','F','H','K','B','I','J','L']),
    ("BCGHIJKL", ['C','G','H','K','B','I','J','L']),
    ("BDEFGHIJ", ['D','F','H','I','B','J','G','E']),
    ("BDEFGHIK", ['D','F','E','K','B','H','G','I']),
    ("BDEFGHIL", ['D','F','E','I','B','H','G','L']),
    ("BDEFGHJK", ['D','F','H','K','B','J','G','E']),
    ("BDEFGHJL", ['D','F','H','E','B','J','G','L']),
    ("BDEFGHKL", ['D','F','E','K','B','H','G','L']),
    ("BDEFGIJK", ['D','F','E','K','B','J','G','I']),
    ("BDEFGIJL", ['D','F','E','I','B','J','G','L']),
    ("BDEFGIKL", ['D','F','E','K','B','I','G','L']),
    ("BDEFGJKL", ['D','F','E','K','B','J','G','L']),
    ("BDEFHIJK", ['D','F','E','K','B','H','J','I']),
    ("BDEFHIJL", ['D','F','E','I','B','H','J','L']),
    ("BDEFHIKL", ['D','F','E','K','B','H','I','L']),
    ("BDEFHJKL", ['D','F','E','K','B','H','J','L']),
    ("BDEFIJKL", ['D','F','E','K','B','I','J','L']),
    ("BDEGHIJK", ['D','G','E','K','B','H','J','I']),
    ("BDEGHIJL", ['D','G','E','I','B','H','J','L']),
    ("BDEGHIKL", ['D','H','E','K','B','I','G','L']),
    ("BDEGHJKL", ['D','G','E','K','B','H','J','L']),
    ("BDEGIJKL", ['D','G','E','K','B','I','J','L']),
    ("BDEHIJKL", ['D','H','E','K','B','I','J','L']),
    ("BDFGHIJK", ['D','F','H','K','B','J','G','I']),
    ("BDFGHIJL", ['D','F','H','I','B','J','G','L']),
    ("BDFGHIKL", ['D','F','H','K','B','I','G','L']),
    ("BDFGHJKL", ['D','F','H','K','B','J','G','L']),
    ("BDFGIJKL", ['D','F','I','K','B','J','G','L']),
    ("BDFHIJKL", ['D','F','H','K','B','I','J','L']),
    ("BDGHIJKL", ['D','G','H','K','B','I','J','L']),
    ("BEFGHIJK", ['F','G','E','K','B','H','J','I']),
    ("BEFGHIJL", ['F','G','E','I','B','H','J','L']),
    ("BEFGHIKL", ['F','H','E','K','B','I','G','L']),
    ("BEFGHJKL", ['F','G','E','K','B','H','J','L']),
    ("BEFGIJKL", ['F','G','E','K','B','I','J','L']),
    ("BEFHIJKL", ['F','H','E','K','B','I','J','L']),
    ("BEGHIJKL", ['B','G','E','K','I','H','J','L']),
    ("BFGHIJKL", ['F','G','H','K','B','I','J','L']),
    ("CDEFGHIJ", ['D','F','C','I','J','H','G','E']),
    ("CDEFGHIK", ['D','F','C','K','E','H','G','I']),
    ("CDEFGHIL", ['D','F','C','I','E','H','G','L']),
    ("CDEFGHJK", ['D','F','C','K','J','H','G','E']),
    ("CDEFGHJL", ['D','F','C','E','J','H','G','L']),
    ("CDEFGHKL", ['D','F','C','K','E','H','G','L']),
    ("CDEFGIJK", ['D','F','C','K','E','J','G','I']),
    ("CDEFGIJL", ['D','F','C','I','E','J','G','L']),
    ("CDEFGIKL", ['D','F','C','K','E','I','G','L']),
    ("CDEFGJKL", ['D','F','C','K','E','J','G','L']),
    ("CDEFHIJK", ['D','F','C','K','E','H','J','I']),
    ("CDEFHIJL", ['D','F','C','I','E','H','J','L']),
    ("CDEFHIKL", ['D','F','C','K','I','H','E','L']),
    ("CDEFHJKL", ['D','F','C','K','E','H','J','L']),
    ("CDEFIJKL", ['D','F','C','K','E','I','J','L']),
    ("CDEGHIJK", ['C','D','E','K','J','H','G','I']),
    ("CDEGHIJL", ['C','D','E','I','J','H','G','L']),
    ("CDEGHIKL", ['C','D','E','K','I','H','G','L']),
    ("CDEGHJKL", ['C','D','E','K','J','H','G','L']),
    ("CDEGIJKL", ['C','D','E','K','I','J','G','L']),
    ("CDEHIJKL", ['C','D','E','K','I','H','J','L']),
    ("CDFGHIJK", ['D','F','C','K','J','H','G','I']),
    ("CDFGHIJL", ['D','F','C','I','J','H','G','L']),
    ("CDFGHIKL", ['D','F','C','K','I','H','G','L']),
    ("CDFGHJKL", ['D','F','C','K','J','H','G','L']),
    ("CDFGIJKL", ['D','F','C','K','I','J','G','L']),
    ("CDFHIJKL", ['D','F','C','K','I','H','J','L']),
    ("CDGHIJKL", ['C','D','H','K','I','J','G','L']),
    ("CEFGHIJK", ['C','F','E','K','J','H','G','I']),
    ("CEFGHIJL", ['C','F','E','I','J','H','G','L']),
    ("CEFGHIKL", ['C','F','E','K','I','H','G','L']),
    ("CEFGHJKL", ['C','F','E','K','J','H','G','L']),
    ("CEFGIJKL", ['C','F','E','K','I','J','G','L']),
    ("CEFHIJKL", ['C','F','E','K','I','H','J','L']),
    ("CEGHIJKL", ['C','G','E','K','I','H','J','L']),
    ("CFGHIJKL", ['C','F','H','K','I','J','G','L']),
    ("DEFGHIJK", ['D','F','E','K','J','H','G','I']),
    ("DEFGHIJL", ['D','F','E','I','J','H','G','L']),
    ("DEFGHIKL", ['D','F','E','K','I','H','G','L']),
    ("DEFGHJKL", ['D','F','E','K','J','H','G','L']),
    ("DEFGIJKL", ['D','F','E','K','I','J','G','L']),
    ("DEFHIJKL", ['D','F','E','K','I','H','J','L']),
    ("DEGHIJKL", ['D','G','E','K','I','H','J','L']),
    ("DFGHIJKL", ['D','F','H','K','I','J','G','L']),
    ("EFGHIJKL", ['F','G','E','K','I','H','J','L']),
];

pub static THIRD_PLACE_TABLE: LazyLock<HashMap<&'static str, [char; 8]>> =
    LazyLock::new(|| THIRD_PLACE_DATA.iter().copied().collect());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_r32_bracket_count() {
        assert_eq!(R32_BRACKET.len(), 16);
    }

    #[test]
    fn test_lookup_returns_official_assignment() {
        // Spot-check a known Annex C row (verified against two independent sources).
        // Value order is [3rd for 1E,1I,1A,1L,1D,1G,1B,1K].
        assert_eq!(
            get_third_place_assignments("ABCDEFGH"),
            Some(['C', 'F', 'H', 'E', 'B', 'A', 'G', 'D'])
        );
        assert!(get_third_place_assignments("EFGHIJKL").is_some());
    }

    #[test]
    fn test_third_place_table_is_complete_and_valid() {
        // Every one of the C(12,8)=495 combinations must be present, and each row
        // must be a bijection that respects the pool constraints. This is the
        // guard that the previously-cleared hand-entered table failed.
        let groups = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L'];
        let mut count = 0;
        for combo in 0u16..(1 << 12) {
            if combo.count_ones() != 8 {
                continue;
            }
            count += 1;
            let key: String = (0..12)
                .filter(|i| combo & (1 << i) != 0)
                .map(|i| groups[i])
                .collect();

            let assignment = get_third_place_assignments(&key)
                .unwrap_or_else(|| panic!("missing Annex C row for {key}"));

            // Bijection: the assigned third-place groups are exactly the key's groups.
            let mut assigned = assignment;
            assigned.sort_unstable();
            let mut expected: Vec<char> = key.chars().collect();
            expected.sort_unstable();
            assert_eq!(assigned.to_vec(), expected, "row {key} is not a bijection");

            // Pool constraints (also enforces no same-group rematch).
            for (slot_idx, &g) in assignment.iter().enumerate() {
                assert!(
                    THIRD_PLACE_POOLS[slot_idx].contains(&g),
                    "row {key}: slot {slot_idx} assigned '{g}' outside pool {:?}",
                    THIRD_PLACE_POOLS[slot_idx]
                );
            }
        }
        assert_eq!(count, 495);
        assert_eq!(THIRD_PLACE_TABLE.len(), 495);
    }

    #[test]
    fn test_all_match_nums_unique() {
        let nums: Vec<u8> = R32_BRACKET.iter().map(|m| m.match_num).collect();
        let mut sorted = nums.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(nums.len(), sorted.len());
    }

    #[test]
    fn test_assignments_respect_pool_constraints() {
        // Test multiple combinations to verify pool constraints are respected
        let test_cases = vec![
            "ABCDEFGH",
            "ABCDEFIJ",
            "EFGHIJKL",
            "ABDEFGJK",
        ];

        for key in test_cases {
            let result = get_third_place_assignments(key);
            assert!(result.is_some(), "Should find assignment for {}", key);
            let assignments = result.unwrap();

            // Verify each assignment respects its pool constraint
            for (slot_idx, &assigned_group) in assignments.iter().enumerate() {
                let pool = &THIRD_PLACE_POOLS[slot_idx];
                assert!(
                    pool.contains(&assigned_group),
                    "Slot {} assigned '{}' but pool is {:?} (key: {})",
                    slot_idx, assigned_group, pool, key
                );
            }
        }
    }
}
