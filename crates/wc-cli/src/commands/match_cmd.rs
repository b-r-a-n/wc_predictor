//! Match prediction command implementation.

use std::collections::HashMap;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::Serialize;

use wc_core::Tournament;
use wc_strategies::{
    CompositeStrategy, EloStrategy, FifaRankingStrategy, MarketValueStrategy, MatchContext,
    PredictionStrategy,
};

use crate::cli::{MatchArgs, OutputFormat, StrategyChoice};
use crate::data::find_team;
use crate::error::{CliError, Result};
use crate::output::{render_match_table, MatchJsonOutput, Output, TeamWithElo};

pub fn run_match(args: &MatchArgs, tournament: &Tournament, format: OutputFormat) -> Result<()> {
    let output = Output::new(format);

    // Find teams
    let team_a = find_team(tournament, &args.team_a)
        .ok_or_else(|| CliError::TeamNotFound(args.team_a.clone()))?;
    let team_b = find_team(tournament, &args.team_b)
        .ok_or_else(|| CliError::TeamNotFound(args.team_b.clone()))?;

    // Create strategy
    let strategy: Box<dyn PredictionStrategy> = match args.strategy {
        StrategyChoice::Elo => Box::new(EloStrategy::default()),
        StrategyChoice::Fifa => Box::new(FifaRankingStrategy::default()),
        StrategyChoice::Market => Box::new(MarketValueStrategy::default()),
        StrategyChoice::Composite => Box::new(
            CompositeStrategy::new("Composite")
                .add_strategy(EloStrategy::default(), 0.4)
                .add_strategy(MarketValueStrategy::default(), 0.3)
                .add_strategy(FifaRankingStrategy::default(), 0.3),
        ),
    };

    // Create match context
    let ctx = MatchContext::new(team_a.clone(), team_b.clone(), args.knockout);

    // Get predictions
    let probs = strategy.predict_probabilities(&ctx);
    let goals = strategy.predict_goals(&ctx);

    // If --simulate is provided, run N match simulations
    if let Some(n) = args.simulate {
        run_match_simulation(args, &ctx, strategy.as_ref(), n, &output)?;
    } else {
        // Just show probabilities
        if output.is_json() {
            let json_output = MatchJsonOutput {
                team_a: TeamWithElo {
                    id: team_a.id.0,
                    name: &team_a.name,
                    code: &team_a.code,
                    elo_rating: team_a.elo_rating,
                },
                team_b: TeamWithElo {
                    id: team_b.id.0,
                    name: &team_b.name,
                    code: &team_b.code,
                    elo_rating: team_b.elo_rating,
                },
                is_knockout: args.knockout,
                probabilities: probs,
                expected_goals: goals,
            };
            output.print_json(&json_output);
        } else {
            render_match_table(team_a, team_b, &probs, &goals, args.knockout);
        }
    }

    Ok(())
}

fn run_match_simulation(
    args: &MatchArgs,
    ctx: &MatchContext,
    strategy: &dyn PredictionStrategy,
    n: u32,
    output: &Output,
) -> Result<()> {
    let seed = args.seed.unwrap_or(42);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    let mut home_wins = 0u32;
    let mut draws = 0u32;
    let mut away_wins = 0u32;
    let mut scorelines: HashMap<(u8, u8), u32> = HashMap::new();

    for _ in 0..n {
        let result = strategy.simulate_match(ctx, &mut rng);
        let (home_goals, away_goals) = (result.home_goals, result.away_goals);

        // Count outcome (accounting for penalties in knockout)
        if let Some(winner) = result.winner() {
            if winner == ctx.home_team.id {
                home_wins += 1;
            } else {
                away_wins += 1;
            }
        } else {
            draws += 1;
        }

        *scorelines.entry((home_goals, away_goals)).or_insert(0) += 1;
    }

    // Sort scorelines by frequency
    let mut sorted_scores: Vec<_> = scorelines.into_iter().collect();
    sorted_scores.sort_by(|a, b| b.1.cmp(&a.1));

    if output.is_json() {
        let json_output = MatchSimulationJsonOutput {
            matches: n,
            home_wins,
            draws,
            away_wins,
            home_win_pct: home_wins as f64 / n as f64,
            draw_pct: draws as f64 / n as f64,
            away_win_pct: away_wins as f64 / n as f64,
            top_scorelines: sorted_scores
                .iter()
                .take(10)
                .map(|((h, a), count)| ScorelineEntry {
                    score: format!("{}-{}", h, a),
                    count: *count,
                    probability: *count as f64 / n as f64,
                })
                .collect(),
        };
        output.print_json(&json_output);
    } else {
        println!();
        println!(
            "Match Simulation: {} vs {} ({} matches)",
            ctx.home_team.name, ctx.away_team.name, n
        );
        println!("{}", "=".repeat(60));
        println!(
            "Match Type: {}",
            if ctx.is_knockout {
                "Knockout"
            } else {
                "Group Stage"
            }
        );
        println!();

        println!("Results Distribution:");
        let bar_width = 30;
        let max_count = home_wins.max(draws).max(away_wins);

        print_bar(
            &format!("{} Win", ctx.home_team.name),
            home_wins,
            n,
            max_count,
            bar_width,
        );
        if !ctx.is_knockout {
            print_bar("Draw", draws, n, max_count, bar_width);
        }
        print_bar(
            &format!("{} Win", ctx.away_team.name),
            away_wins,
            n,
            max_count,
            bar_width,
        );

        println!();
        println!("Most Common Scorelines:");
        for ((h, a), count) in sorted_scores.iter().take(5) {
            println!(
                "  {}-{}: {} ({:.1}%)",
                h,
                a,
                count,
                *count as f64 / n as f64 * 100.0
            );
        }
        println!();
    }

    Ok(())
}

fn print_bar(label: &str, count: u32, total: u32, max: u32, width: usize) {
    let pct = count as f64 / total as f64 * 100.0;
    let bar_len = if max > 0 {
        (count as f64 / max as f64 * width as f64) as usize
    } else {
        0
    };
    let bar: String = "█".repeat(bar_len);
    println!("  {:20} {:5} ({:5.1}%) {}", label, count, pct, bar);
}

#[derive(Serialize)]
struct MatchSimulationJsonOutput {
    matches: u32,
    home_wins: u32,
    draws: u32,
    away_wins: u32,
    home_win_pct: f64,
    draw_pct: f64,
    away_win_pct: f64,
    top_scorelines: Vec<ScorelineEntry>,
}

#[derive(Serialize)]
struct ScorelineEntry {
    score: String,
    count: u32,
    probability: f64,
}
