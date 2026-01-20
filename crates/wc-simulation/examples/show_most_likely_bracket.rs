//! Display the most likely bracket from 100k simulations.

use std::collections::HashMap;
use wc_simulation::{SimulationConfig, SimulationRunner};
use wc_strategies::EloStrategy;
use wc_core::bracket::{R32_BRACKET, SlotSource, GroupPosition};
use wc_core::TeamId;

fn main() {
    // Load tournament data
    let data = include_str!("../../../data/teams.json");
    let tournament: wc_core::Tournament = serde_json::from_str(data).unwrap();

    let strategy = EloStrategy::default();
    let config = SimulationConfig::with_iterations(100_000).with_seed(42);
    let runner = SimulationRunner::new(&tournament, &strategy, config);

    println!("Running 100,000 simulations...\n");
    let results = runner.run();

    let bracket = &results.most_likely_bracket;

    // Build team-to-group mapping
    let mut team_to_group: HashMap<TeamId, char> = HashMap::new();
    for group in &tournament.groups {
        for &team_id in &group.teams {
            team_to_group.insert(team_id, group.id.0);
        }
    }

    println!("════════════════════════════════════════════════════════════════════════════════");
    println!("                    MOST LIKELY BRACKET (100k sims)");
    println!("════════════════════════════════════════════════════════════════════════════════\n");

    // Find most likely opponent for each R32 winner
    let mut r32_matchups: Vec<(TeamId, TeamId)> = Vec::new(); // (winner, loser)

    println!("ROUND OF 32 - ALL 32 PARTICIPANTS:");
    println!("────────────────────────────────────────────────────────────────────────────────");
    for slot in 0..16u8 {
        let r32_match = &R32_BRACKET[slot as usize];

        if let Some(winner_data) = bracket.round_of_32.get(&slot) {
            let winner = tournament.get_team(winner_data.team_id).unwrap();

            // Find the most likely opponent for this winner at this slot
            let opponent_id = if let Some(opp_stats) = results.slot_opponent_stats.get(&winner_data.team_id) {
                // Get opponents at this R32 slot
                opp_stats.round_of_32.get(&slot)
                    .and_then(|opponents| {
                        opponents.iter()
                            .max_by_key(|(_, count)| *count)
                            .map(|(opp_id, _)| *opp_id)
                    })
            } else {
                None
            };

            if let Some(opp_id) = opponent_id {
                let opponent = tournament.get_team(opp_id).unwrap();
                r32_matchups.push((winner_data.team_id, opp_id));

                println!("  M{:2} (slot {:2}): {:>12} ({:>3}) vs {:<12} ({:<3})  → Winner: {}",
                    r32_match.match_num, slot,
                    winner.name, winner.code,
                    opponent.name, opponent.code,
                    winner.code);
            } else {
                r32_matchups.push((winner_data.team_id, winner_data.team_id)); // placeholder
                println!("  M{:2} (slot {:2}): {:>12} ({:>3}) vs ???                    → Winner: {}",
                    r32_match.match_num, slot,
                    winner.name, winner.code, winner.code);
            }
        } else {
            println!("  M{:2} (slot {:2}): <no data>", r32_match.match_num, slot);
        }
    }

    // Collect all 32 unique R32 participants
    let mut all_r32_teams: Vec<TeamId> = Vec::new();
    for (winner, loser) in &r32_matchups {
        if !all_r32_teams.contains(winner) {
            all_r32_teams.push(*winner);
        }
        if *loser != *winner && !all_r32_teams.contains(loser) {
            all_r32_teams.push(*loser);
        }
    }

    println!("\n  Total unique R32 participants: {}", all_r32_teams.len());

    // R16 Winners
    println!("\nROUND OF 16 WINNERS:");
    println!("─────────────────────────────────────────────────────────────────────");
    for slot in 0..8u8 {
        if let Some(slot_data) = bracket.round_of_16.get(&slot) {
            let team = tournament.get_team(slot_data.team_id).unwrap();
            println!("  R16 slot {}: {:>15} ({:>3}) - {:.1}% ({} wins)",
                slot, team.name, team.code,
                slot_data.probability * 100.0, slot_data.count);
        } else {
            println!("  R16 slot {}: <no data>", slot);
        }
    }

    // QF Winners
    println!("\nQUARTER-FINAL WINNERS:");
    println!("─────────────────────────────────────────────────────────────────────");
    for slot in 0..4u8 {
        if let Some(slot_data) = bracket.quarter_finals.get(&slot) {
            let team = tournament.get_team(slot_data.team_id).unwrap();
            println!("  QF slot {}: {:>15} ({:>3}) - {:.1}% ({} wins)",
                slot, team.name, team.code,
                slot_data.probability * 100.0, slot_data.count);
        } else {
            println!("  QF slot {}: <no data>", slot);
        }
    }

    // SF Winners
    println!("\nSEMI-FINAL WINNERS:");
    println!("─────────────────────────────────────────────────────────────────────");
    for slot in 0..2u8 {
        if let Some(slot_data) = bracket.semi_finals.get(&slot) {
            let team = tournament.get_team(slot_data.team_id).unwrap();
            println!("  SF slot {}: {:>15} ({:>3}) - {:.1}% ({} wins)",
                slot, team.name, team.code,
                slot_data.probability * 100.0, slot_data.count);
        } else {
            println!("  SF slot {}: <no data>", slot);
        }
    }

    // Champion
    println!("\nCHAMPION:");
    println!("─────────────────────────────────────────────────────────────────────");
    if let Some(slot_data) = &bracket.champion {
        let team = tournament.get_team(slot_data.team_id).unwrap();
        println!("  🏆 {:>15} ({:>3}) - {:.1}% ({} wins)",
            team.name, team.code,
            slot_data.probability * 100.0, slot_data.count);
    }

    // Verify bracket validity
    println!("\n════════════════════════════════════════════════════════════════════════════════");
    println!("                       BRACKET VALIDATION - ALL 32 R32 TEAMS");
    println!("════════════════════════════════════════════════════════════════════════════════\n");

    let mut validation_errors = Vec::new();

    // Third place pools for reference
    let third_place_pools: [[char; 5]; 8] = [
        ['A', 'B', 'C', 'D', 'F'],  // slot 0: 1E opponent
        ['C', 'D', 'F', 'G', 'H'],  // slot 1: 1I opponent
        ['C', 'E', 'F', 'H', 'I'],  // slot 2: 1A opponent
        ['E', 'H', 'I', 'J', 'K'],  // slot 3: 1L opponent
        ['B', 'E', 'F', 'I', 'J'],  // slot 4: 1D opponent
        ['A', 'E', 'H', 'I', 'J'],  // slot 5: 1G opponent
        ['E', 'F', 'G', 'I', 'J'],  // slot 6: 1B opponent
        ['D', 'E', 'I', 'J', 'L'],  // slot 7: 1K opponent
    ];

    // Helper to check if a team can appear at a slot source
    let can_team_be_at_source = |team_id: TeamId, source: &SlotSource| -> (bool, String) {
        let team_group = team_to_group.get(&team_id).copied().unwrap_or('?');
        let team = tournament.get_team(team_id).unwrap();

        match source {
            SlotSource::GroupTeam { group, position } => {
                let pos_str = match position {
                    GroupPosition::Winner => "1",
                    GroupPosition::RunnerUp => "2",
                    GroupPosition::Third => "3",
                };
                if team_group == *group {
                    (true, format!("{}{} ({})", pos_str, group, team.code))
                } else {
                    (false, format!("{} in Group {} cannot be {}{}", team.code, team_group, pos_str, group))
                }
            }
            SlotSource::ThirdPlacePool { slot_index } => {
                let pool = &third_place_pools[*slot_index as usize];
                if pool.contains(&team_group) {
                    (true, format!("3rd from {} (pool {:?})", team_group, pool))
                } else {
                    (false, format!("{} in Group {} not in 3rd-place pool {:?}", team.code, team_group, pool))
                }
            }
        }
    };

    println!("Verifying all 32 R32 participants against bracket constraints:\n");

    for (slot, (winner_id, loser_id)) in r32_matchups.iter().enumerate() {
        let slot_u8 = slot as u8;
        let r32_match = &R32_BRACKET[slot];
        let winner = tournament.get_team(*winner_id).unwrap();

        println!("  M{:2} (slot {:2}):", r32_match.match_num, slot);

        // Verify winner can be at team_a or team_b position
        let (winner_valid_a, desc_a) = can_team_be_at_source(*winner_id, &r32_match.team_a);
        let (winner_valid_b, desc_b) = can_team_be_at_source(*winner_id, &r32_match.team_b);

        if winner_valid_a {
            println!("    Winner {:>3} ({:>12}): ✓ Can be {} (team_a)", winner.code, winner.name, desc_a);
        } else if winner_valid_b {
            println!("    Winner {:>3} ({:>12}): ✓ Can be {} (team_b)", winner.code, winner.name, desc_b);
        } else {
            println!("    Winner {:>3} ({:>12}): ❌ INVALID", winner.code, winner.name);
            validation_errors.push(format!("M{} winner {} invalid: {} / {}", r32_match.match_num, winner.code, desc_a, desc_b));
        }

        // Verify loser can be at the opposite position
        if *loser_id != *winner_id {
            let loser = tournament.get_team(*loser_id).unwrap();
            let (loser_valid_a, loser_desc_a) = can_team_be_at_source(*loser_id, &r32_match.team_a);
            let (loser_valid_b, loser_desc_b) = can_team_be_at_source(*loser_id, &r32_match.team_b);

            // Loser should be at the opposite position from winner
            if winner_valid_a && loser_valid_b {
                println!("    Loser  {:>3} ({:>12}): ✓ Can be {} (team_b)", loser.code, loser.name, loser_desc_b);
            } else if winner_valid_b && loser_valid_a {
                println!("    Loser  {:>3} ({:>12}): ✓ Can be {} (team_a)", loser.code, loser.name, loser_desc_a);
            } else if loser_valid_a || loser_valid_b {
                // Loser is valid for at least one position
                let desc = if loser_valid_a { loser_desc_a } else { loser_desc_b };
                println!("    Loser  {:>3} ({:>12}): ✓ Can be {}", loser.code, loser.name, desc);
            } else {
                println!("    Loser  {:>3} ({:>12}): ❌ INVALID", loser.code, loser.name);
                validation_errors.push(format!("M{} loser {} invalid: {} / {}", r32_match.match_num, loser.code, loser_desc_a, loser_desc_b));
            }
        }
    }

    // Check R16 winners come from R32 feeders
    println!("\nVerifying R16/QF/SF/Final bracket flow:\n");
    for slot in 0..8u8 {
        if let Some(r16_data) = bracket.round_of_16.get(&slot) {
            let feeder1 = slot * 2;
            let feeder2 = slot * 2 + 1;
            let r32_team1 = bracket.round_of_32.get(&feeder1).map(|s| s.team_id);
            let r32_team2 = bracket.round_of_32.get(&feeder2).map(|s| s.team_id);

            if r32_team1 != Some(r16_data.team_id) && r32_team2 != Some(r16_data.team_id) {
                let team = tournament.get_team(r16_data.team_id).unwrap();
                validation_errors.push(format!(
                    "R16 slot {}: {} not from R32 feeders {} or {}",
                    slot, team.code, feeder1, feeder2
                ));
            }
        }
    }

    // Check QF winners come from R16 feeders
    for slot in 0..4u8 {
        if let Some(qf_data) = bracket.quarter_finals.get(&slot) {
            let feeder1 = slot * 2;
            let feeder2 = slot * 2 + 1;
            let r16_team1 = bracket.round_of_16.get(&feeder1).map(|s| s.team_id);
            let r16_team2 = bracket.round_of_16.get(&feeder2).map(|s| s.team_id);

            if r16_team1 != Some(qf_data.team_id) && r16_team2 != Some(qf_data.team_id) {
                let team = tournament.get_team(qf_data.team_id).unwrap();
                validation_errors.push(format!(
                    "QF slot {}: {} not from R16 feeders {} or {}",
                    slot, team.code, feeder1, feeder2
                ));
            }
        }
    }

    // Check SF winners come from QF feeders
    for slot in 0..2u8 {
        if let Some(sf_data) = bracket.semi_finals.get(&slot) {
            let feeder1 = slot * 2;
            let feeder2 = slot * 2 + 1;
            let qf_team1 = bracket.quarter_finals.get(&feeder1).map(|s| s.team_id);
            let qf_team2 = bracket.quarter_finals.get(&feeder2).map(|s| s.team_id);

            if qf_team1 != Some(sf_data.team_id) && qf_team2 != Some(sf_data.team_id) {
                let team = tournament.get_team(sf_data.team_id).unwrap();
                validation_errors.push(format!(
                    "SF slot {}: {} not from QF feeders {} or {}",
                    slot, team.code, feeder1, feeder2
                ));
            }
        }
    }

    // Check champion comes from SF feeders
    if let Some(champion_data) = &bracket.champion {
        let sf_team1 = bracket.semi_finals.get(&0).map(|s| s.team_id);
        let sf_team2 = bracket.semi_finals.get(&1).map(|s| s.team_id);

        if sf_team1 != Some(champion_data.team_id) && sf_team2 != Some(champion_data.team_id) {
            let team = tournament.get_team(champion_data.team_id).unwrap();
            validation_errors.push(format!(
                "Champion {} not from SF feeders",
                team.code
            ));
        }
    }

    // Check no duplicate teams among all 32 R32 participants
    let unique_count = {
        let mut unique: std::collections::HashSet<_> = std::collections::HashSet::new();
        for t in &all_r32_teams {
            unique.insert(*t);
        }
        unique.len()
    };

    println!("\n────────────────────────────────────────────────────────────────────────────────");
    if validation_errors.is_empty() {
        println!("✅ All bracket POSITION constraints validated!");
        println!("   - All {} R32 participants can appear in their assigned positions", all_r32_teams.len());
        println!("   - R16 winners come from correct R32 feeder matches");
        println!("   - QF winners come from correct R16 feeder matches");
        println!("   - SF winners come from correct QF feeder matches");
        println!("   - Champion comes from correct SF feeder matches");
    } else {
        println!("❌ Bracket validation errors:");
        for err in &validation_errors {
            println!("   - {}", err);
        }
    }

    // Note about unique teams and identify duplicates
    if unique_count < 32 {
        println!("\n⚠️  NOTE: Only {} unique teams shown (not 32)", unique_count);
        println!("   This is expected! The 'most likely bracket' computes:");
        println!("   - 16 unique WINNERS (enforced by greedy algorithm)");
        println!("   - 16 OPPONENTS computed independently per match");
        println!("");
        println!("   Some teams appear in multiple matches because they have high");
        println!("   probability of reaching R32 via different group positions:");
        println!("   - A team could be 1st, 2nd, OR 3rd in their group");
        println!("   - Different group finishes lead to different R32 slots");

        // Find and display duplicate teams
        let mut team_appearances: HashMap<TeamId, Vec<(usize, &str)>> = HashMap::new();
        for (slot, (winner, loser)) in r32_matchups.iter().enumerate() {
            team_appearances.entry(*winner).or_default().push((slot, "winner"));
            if winner != loser {
                team_appearances.entry(*loser).or_default().push((slot, "loser"));
            }
        }

        let duplicates: Vec<_> = team_appearances.iter()
            .filter(|(_, appearances)| appearances.len() > 1)
            .collect();

        if !duplicates.is_empty() {
            println!("\n   Teams appearing in multiple R32 matches:");
            for (team_id, appearances) in duplicates {
                let team = tournament.get_team(*team_id).unwrap();
                let group = team_to_group.get(team_id).unwrap();
                let slots: Vec<String> = appearances.iter()
                    .map(|(slot, role)| format!("M{} as {}", R32_BRACKET[*slot].match_num, role))
                    .collect();
                println!("   - {} (Group {}): {}", team.code, group, slots.join(", "));
            }
        }

        println!("");
        println!("   Each team-position pairing IS VALID per FIFA bracket rules.");
        println!("   However, this is NOT a single coherent tournament outcome.");
        println!("   For that, see 'most_frequent_bracket' (much lower probability).");
    }

    // Show R32 slot mapping for reference
    println!("\n════════════════════════════════════════════════════════════════════");
    println!("                    R32 SLOT MAPPING REFERENCE");
    println!("════════════════════════════════════════════════════════════════════\n");

    for (i, r32_match) in R32_BRACKET.iter().enumerate() {
        let team_a_desc = match &r32_match.team_a {
            wc_core::bracket::SlotSource::GroupTeam { group, position } => {
                format!("{}{}", match position {
                    wc_core::bracket::GroupPosition::Winner => "1",
                    wc_core::bracket::GroupPosition::RunnerUp => "2",
                    wc_core::bracket::GroupPosition::Third => "3",
                }, group)
            }
            wc_core::bracket::SlotSource::ThirdPlacePool { slot_index } => {
                format!("3rd(pool {})", slot_index)
            }
        };
        let team_b_desc = match &r32_match.team_b {
            wc_core::bracket::SlotSource::GroupTeam { group, position } => {
                format!("{}{}", match position {
                    wc_core::bracket::GroupPosition::Winner => "1",
                    wc_core::bracket::GroupPosition::RunnerUp => "2",
                    wc_core::bracket::GroupPosition::Third => "3",
                }, group)
            }
            wc_core::bracket::SlotSource::ThirdPlacePool { slot_index } => {
                format!("3rd(pool {})", slot_index)
            }
        };
        println!("  Slot {:2} (M{:2}): {:12} vs {:12}",
            i, r32_match.match_num, team_a_desc, team_b_desc);
    }
}
