//! Display R32 bracket for a single simulation.

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use wc_core::bracket::R32_BRACKET;
use wc_simulation::SimulationEngine;
use wc_strategies::EloStrategy;

fn main() {
    // Load tournament data
    let data = include_str!("../../../data/teams.json");
    let tournament: wc_core::Tournament = serde_json::from_str(data).unwrap();

    let strategy = EloStrategy::default();
    let engine = SimulationEngine::new(&tournament, &strategy);

    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let result = engine.simulate(&mut rng);

    println!("\n════════════════════════════════════════════");
    println!("         GROUP STAGE RESULTS");
    println!("════════════════════════════════════════════\n");

    for gr in &result.group_results {
        let w = tournament.get_team(gr.winner()).unwrap();
        let r = tournament.get_team(gr.runner_up()).unwrap();
        let t = tournament.get_team(gr.third_place()).unwrap();
        println!("Group {}: 1st {:3} | 2nd {:3} | 3rd {:3}",
            gr.group_id.0, w.code, r.code, t.code);
    }

    // Show qualifying third-place teams
    println!("\n════════════════════════════════════════════");
    println!("      QUALIFYING THIRD-PLACE TEAMS");
    println!("════════════════════════════════════════════\n");

    let third_standings: Vec<_> = result.group_results
        .iter()
        .map(|gr| (gr.group_id.0, gr.third_place(), gr.standings[2].points, gr.standings[2].goal_difference()))
        .collect();

    // Sort by points, then GD (simplified ranking)
    let mut sorted_thirds = third_standings.clone();
    sorted_thirds.sort_by(|a, b| {
        b.2.cmp(&a.2).then_with(|| b.3.cmp(&a.3))
    });

    println!("All 12 third-place teams (sorted by points/GD):");
    for (i, (group, team_id, pts, gd)) in sorted_thirds.iter().enumerate() {
        let team = tournament.get_team(*team_id).unwrap();
        let qualified = if i < 8 { "✓" } else { " " };
        println!("  {} Group {}: {:3} - {} pts, {} GD", qualified, group, team.code, pts, gd);
    }

    println!("\n════════════════════════════════════════════");
    println!("           ROUND OF 32 BRACKET");
    println!("════════════════════════════════════════════\n");

    for (i, m) in result.knockout_bracket.round_of_32.iter().enumerate() {
        let home = tournament.get_team(m.home_team).unwrap();
        let away = tournament.get_team(m.away_team).unwrap();
        let winner_code = m.winner()
            .map(|w| tournament.get_team(w).unwrap().code.clone())
            .unwrap_or_else(|| "?".to_string());

        println!("Match {:2}: {:12} {:>3} {} - {} {:<3} {:12}  → {}",
            R32_BRACKET[i].match_num,
            home.name, home.code,
            m.home_goals, m.away_goals,
            away.code, away.name,
            winner_code
        );
    }

    println!("\n════════════════════════════════════════════");
    println!("              FINAL RESULTS");
    println!("════════════════════════════════════════════\n");

    let champion = tournament.get_team(result.champion).unwrap();
    let runner_up = tournament.get_team(result.runner_up).unwrap();
    let third = tournament.get_team(result.third_place).unwrap();
    let fourth = tournament.get_team(result.fourth_place).unwrap();

    println!("🥇 Champion:    {}", champion.name);
    println!("🥈 Runner-up:   {}", runner_up.name);
    println!("🥉 Third place: {}", third.name);
    println!("4th place:      {}", fourth.name);
}
