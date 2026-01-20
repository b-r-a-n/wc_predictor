//! Display the optimal bracket from 100k simulations using the Hungarian algorithm.

use std::collections::HashSet;
use wc_simulation::{SimulationConfig, SimulationRunner};
use wc_strategies::EloStrategy;
use wc_simulation::optimal_bracket::verify_optimal_bracket;

fn main() {
    // Load tournament data
    let data = include_str!("../../../data/teams.json");
    let tournament: wc_core::Tournament = serde_json::from_str(data).unwrap();

    let strategy = EloStrategy::default();
    let config = SimulationConfig::with_iterations(100_000).with_seed(42);
    let runner = SimulationRunner::new(&tournament, &strategy, config);

    println!("Running 100,000 simulations...\n");
    let results = runner.run();

    let bracket = &results.optimal_bracket;

    println!("════════════════════════════════════════════════════════════════════════════════");
    println!("           OPTIMAL BRACKET (Hungarian Algorithm - 100k sims)");
    println!("════════════════════════════════════════════════════════════════════════════════\n");

    // Show R32 matches with both teams
    println!("ROUND OF 32 - ALL 32 PARTICIPANTS (16 matches):");
    println!("────────────────────────────────────────────────────────────────────────────────");

    let mut all_teams: HashSet<wc_core::TeamId> = HashSet::new();

    for m in &bracket.round_of_32 {
        let team_a = tournament.get_team(m.team_a.team_id).unwrap();
        let team_b = tournament.get_team(m.team_b.team_id).unwrap();
        let winner = tournament.get_team(m.winner).unwrap();

        all_teams.insert(m.team_a.team_id);
        all_teams.insert(m.team_b.team_id);

        let winner_marker_a = if m.winner == m.team_a.team_id { "→" } else { " " };
        let winner_marker_b = if m.winner == m.team_b.team_id { "→" } else { " " };

        println!(
            "  Slot {:2}: {} {:>12} ({:>3}) {:.1}%  vs  {:<12} ({:<3}) {:.1}% {}  │ Winner: {}",
            m.slot,
            winner_marker_a,
            team_a.name,
            team_a.code,
            m.team_a.probability * 100.0,
            team_b.name,
            team_b.code,
            m.team_b.probability * 100.0,
            winner_marker_b,
            winner.code
        );
    }

    println!("\n  Total unique R32 participants: {}", all_teams.len());

    // R16 Winners
    println!("\nROUND OF 16 WINNERS:");
    println!("─────────────────────────────────────────────────────────────────────");
    for slot in 0..8u8 {
        if let Some(slot_data) = bracket.round_of_16.get(&slot) {
            let team = tournament.get_team(slot_data.team_id).unwrap();
            println!(
                "  R16 slot {}: {:>15} ({:>3}) - {:.1}% ({} wins)",
                slot,
                team.name,
                team.code,
                slot_data.probability * 100.0,
                slot_data.count
            );
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
            println!(
                "  QF slot {}: {:>15} ({:>3}) - {:.1}% ({} wins)",
                slot,
                team.name,
                team.code,
                slot_data.probability * 100.0,
                slot_data.count
            );
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
            println!(
                "  SF slot {}: {:>15} ({:>3}) - {:.1}% ({} wins)",
                slot,
                team.name,
                team.code,
                slot_data.probability * 100.0,
                slot_data.count
            );
        } else {
            println!("  SF slot {}: <no data>", slot);
        }
    }

    // Champion
    println!("\nCHAMPION:");
    println!("─────────────────────────────────────────────────────────────────────");
    if let Some(slot_data) = &bracket.champion {
        let team = tournament.get_team(slot_data.team_id).unwrap();
        println!(
            "  🏆 {:>15} ({:>3}) - {:.1}% ({} wins)",
            team.name,
            team.code,
            slot_data.probability * 100.0,
            slot_data.count
        );
    }

    // Joint probability
    println!("\nPROBABILITY METRICS:");
    println!("─────────────────────────────────────────────────────────────────────");
    println!("  Joint probability: {:.2e}", bracket.joint_probability);
    println!("  Log probability:   {:.2}", bracket.log_probability);

    // Verification
    println!("\n════════════════════════════════════════════════════════════════════════════════");
    println!("                              VERIFICATION");
    println!("════════════════════════════════════════════════════════════════════════════════\n");

    match verify_optimal_bracket(bracket) {
        Ok(()) => {
            println!("✅ Optimal bracket is VALID:");
            println!("   - Exactly 32 unique teams in R32");
            println!("   - All assignments satisfy eligibility constraints");
        }
        Err(errors) => {
            println!("❌ Bracket validation errors:");
            for err in errors {
                println!("   - {}", err);
            }
        }
    }

    // Compare with greedy bracket
    println!("\n════════════════════════════════════════════════════════════════════════════════");
    println!("                  COMPARISON WITH GREEDY BRACKET");
    println!("════════════════════════════════════════════════════════════════════════════════\n");

    let greedy = &results.most_likely_bracket;
    let greedy_teams: HashSet<_> = greedy.round_of_32.values().map(|s| s.team_id).collect();

    println!("  Greedy bracket R32 unique teams: {}", greedy_teams.len());
    println!("  Optimal bracket R32 unique teams: {}", all_teams.len());

    if greedy_teams.len() < 16 {
        println!("\n  ⚠️  Greedy bracket has missing slots (expected 16 R32 winners)");
    }
    if all_teams.len() < 32 {
        println!("\n  ⚠️  Optimal bracket has fewer than 32 teams (assignment failed for some slots)");
    }
}
