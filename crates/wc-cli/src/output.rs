//! Output formatting for table and JSON modes.

use comfy_table::{presets::UTF8_FULL, Cell, ContentArrangement, Table};
use serde::Serialize;
use wc_core::{Team, Tournament};
use wc_simulation::AggregatedResults;
use wc_strategies::{GoalExpectation, MatchProbabilities};

use crate::cli::OutputFormat;

/// Output handler based on format selection.
pub struct Output {
    format: OutputFormat,
}

impl Output {
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    pub fn is_json(&self) -> bool {
        matches!(self.format, OutputFormat::Json)
    }

    pub fn print_json<T: Serialize>(&self, data: &T) {
        println!("{}", serde_json::to_string_pretty(data).unwrap());
    }
}

/// Render simulation results as a table.
pub fn render_simulation_table(results: &AggregatedResults, top_n: usize, tournament: &Tournament) {
    let rankings = results.top_n(top_n);

    println!();
    println!(
        "World Cup 2026 Simulation Results ({} iterations)",
        results.total_simulations
    );
    println!("{}", "=".repeat(60));

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Rank"),
            Cell::new("Team"),
            Cell::new("Win %"),
            Cell::new("Final %"),
            Cell::new("Semi %"),
            Cell::new("Knockout %"),
        ]);

    for (i, stats) in rankings.iter().enumerate() {
        table.add_row(vec![
            Cell::new(i + 1),
            Cell::new(&stats.team_name),
            Cell::new(format!("{:.2}%", stats.win_probability * 100.0)),
            Cell::new(format!("{:.2}%", stats.final_probability * 100.0)),
            Cell::new(format!("{:.2}%", stats.semi_final_probability * 100.0)),
            Cell::new(format!("{:.2}%", stats.knockout_probability * 100.0)),
        ]);
    }

    println!("{table}");

    // Most likely final
    let winner = tournament.get_team(results.most_likely_winner);
    let final_team_a = tournament.get_team(results.most_likely_final.0);
    let final_team_b = tournament.get_team(results.most_likely_final.1);

    println!();
    if let Some(w) = winner {
        println!("Most Likely Winner: {}", w.name);
    }
    if let (Some(a), Some(b)) = (final_team_a, final_team_b) {
        println!("Most Likely Final: {} vs {}", a.name, b.name);
    }
    println!();
}

/// Render simulation results as JSON.
#[derive(Serialize)]
pub struct SimulationJsonOutput<'a> {
    pub total_simulations: u32,
    pub most_likely_winner: TeamSummary<'a>,
    pub most_likely_final: FinalMatchup<'a>,
    pub rankings: Vec<TeamRanking<'a>>,
}

#[derive(Serialize)]
pub struct TeamSummary<'a> {
    pub id: u8,
    pub name: &'a str,
    pub code: &'a str,
}

#[derive(Serialize)]
pub struct FinalMatchup<'a> {
    pub team_a: TeamSummary<'a>,
    pub team_b: TeamSummary<'a>,
}

#[derive(Serialize)]
pub struct TeamRanking<'a> {
    pub rank: usize,
    pub team: TeamSummary<'a>,
    pub win_probability: f64,
    pub final_probability: f64,
    pub semi_final_probability: f64,
    pub knockout_probability: f64,
    pub champion_count: u32,
    pub final_count: u32,
}

impl<'a> SimulationJsonOutput<'a> {
    pub fn from_results(results: &AggregatedResults, tournament: &'a Tournament) -> Self {
        let winner = tournament.get_team(results.most_likely_winner).unwrap();
        let final_a = tournament.get_team(results.most_likely_final.0).unwrap();
        let final_b = tournament.get_team(results.most_likely_final.1).unwrap();

        let rankings: Vec<TeamRanking> = results
            .rankings()
            .iter()
            .enumerate()
            .map(|(i, stats)| {
                let team = tournament.get_team(stats.team_id).unwrap();
                TeamRanking {
                    rank: i + 1,
                    team: TeamSummary {
                        id: team.id.0,
                        name: &team.name,
                        code: &team.code,
                    },
                    win_probability: stats.win_probability,
                    final_probability: stats.final_probability,
                    semi_final_probability: stats.semi_final_probability,
                    knockout_probability: stats.knockout_probability,
                    champion_count: stats.champion,
                    final_count: stats.reached_final,
                }
            })
            .collect();

        Self {
            total_simulations: results.total_simulations,
            most_likely_winner: TeamSummary {
                id: winner.id.0,
                name: &winner.name,
                code: &winner.code,
            },
            most_likely_final: FinalMatchup {
                team_a: TeamSummary {
                    id: final_a.id.0,
                    name: &final_a.name,
                    code: &final_a.code,
                },
                team_b: TeamSummary {
                    id: final_b.id.0,
                    name: &final_b.name,
                    code: &final_b.code,
                },
            },
            rankings,
        }
    }
}

/// Render match prediction as table.
pub fn render_match_table(
    team_a: &Team,
    team_b: &Team,
    probs: &MatchProbabilities,
    goals: &GoalExpectation,
    is_knockout: bool,
) {
    println!();
    println!(
        "Match Prediction: {} vs {}",
        team_a.name, team_b.name
    );
    println!("{}", "=".repeat(50));
    println!(
        "Match Type: {}",
        if is_knockout { "Knockout" } else { "Group Stage" }
    );
    println!();

    println!("ELO Ratings:");
    println!("  {}: {:.0}", team_a.name, team_a.elo_rating);
    println!("  {}: {:.0}", team_b.name, team_b.elo_rating);
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![Cell::new("Outcome"), Cell::new("Probability")]);

    table.add_row(vec![
        Cell::new(format!("{} Win", team_a.name)),
        Cell::new(format!("{:.2}%", probs.home_win * 100.0)),
    ]);

    if !is_knockout {
        table.add_row(vec![
            Cell::new("Draw"),
            Cell::new(format!("{:.2}%", probs.draw * 100.0)),
        ]);
    }

    table.add_row(vec![
        Cell::new(format!("{} Win", team_b.name)),
        Cell::new(format!("{:.2}%", probs.away_win * 100.0)),
    ]);

    println!("{table}");

    println!();
    println!("Expected Goals:");
    println!("  {}: {:.2}", team_a.name, goals.home_lambda);
    println!("  {}: {:.2}", team_b.name, goals.away_lambda);
    println!();
}

/// JSON output for match prediction.
#[derive(Serialize)]
pub struct MatchJsonOutput<'a> {
    pub team_a: TeamWithElo<'a>,
    pub team_b: TeamWithElo<'a>,
    pub is_knockout: bool,
    pub probabilities: MatchProbabilities,
    pub expected_goals: GoalExpectation,
}

#[derive(Serialize)]
pub struct TeamWithElo<'a> {
    pub id: u8,
    pub name: &'a str,
    pub code: &'a str,
    pub elo_rating: f64,
}

/// Render team info as table.
pub fn render_team_info_table(team: &Team, group_id: Option<char>, group_opponents: &[&Team]) {
    println!();
    println!("Team Information: {}", team.name);
    println!("{}", "=".repeat(40));

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![Cell::new("Field"), Cell::new("Value")]);

    table.add_row(vec![Cell::new("Name"), Cell::new(&team.name)]);
    table.add_row(vec![Cell::new("Code"), Cell::new(&team.code)]);
    table.add_row(vec![
        Cell::new("Confederation"),
        Cell::new(format!("{:?}", team.confederation)),
    ]);
    table.add_row(vec![
        Cell::new("ELO Rating"),
        Cell::new(format!("{:.0}", team.elo_rating)),
    ]);
    table.add_row(vec![
        Cell::new("FIFA Ranking"),
        Cell::new(team.fifa_ranking.to_string()),
    ]);
    table.add_row(vec![
        Cell::new("Market Value"),
        Cell::new(format!("{:.1}M", team.market_value_millions)),
    ]);
    table.add_row(vec![
        Cell::new("World Cup Wins"),
        Cell::new(team.world_cup_wins.to_string()),
    ]);

    if let Some(g) = group_id {
        table.add_row(vec![Cell::new("Group"), Cell::new(g.to_string())]);
    }

    if !group_opponents.is_empty() {
        let opponents: Vec<&str> = group_opponents.iter().map(|t| t.name.as_str()).collect();
        table.add_row(vec![
            Cell::new("Group Opponents"),
            Cell::new(opponents.join(", ")),
        ]);
    }

    println!("{table}");
    println!();
}

/// JSON output for team info.
#[derive(Serialize)]
pub struct TeamInfoJsonOutput<'a> {
    pub id: u8,
    pub name: &'a str,
    pub code: &'a str,
    pub confederation: &'a str,
    pub elo_rating: f64,
    pub fifa_ranking: u16,
    pub market_value_millions: f64,
    pub world_cup_wins: u8,
    pub group: Option<char>,
    pub group_opponents: Vec<TeamSummary<'a>>,
}

/// Render teams list as table.
pub fn render_teams_table(teams: &[&Team]) {
    println!();
    println!("All Teams ({} total)", teams.len());
    println!("{}", "=".repeat(60));

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("#"),
            Cell::new("Name"),
            Cell::new("Code"),
            Cell::new("Conf"),
            Cell::new("ELO"),
            Cell::new("FIFA"),
            Cell::new("Value (M)"),
        ]);

    for (i, team) in teams.iter().enumerate() {
        table.add_row(vec![
            Cell::new(i + 1),
            Cell::new(&team.name),
            Cell::new(&team.code),
            Cell::new(format!("{:?}", team.confederation)),
            Cell::new(format!("{:.0}", team.elo_rating)),
            Cell::new(team.fifa_ranking.to_string()),
            Cell::new(format!("{:.1}", team.market_value_millions)),
        ]);
    }

    println!("{table}");
    println!();
}

/// JSON output for teams list.
#[derive(Serialize)]
pub struct TeamsListJsonOutput<'a> {
    pub total: usize,
    pub teams: Vec<TeamFullInfo<'a>>,
}

#[derive(Serialize)]
pub struct TeamFullInfo<'a> {
    pub id: u8,
    pub name: &'a str,
    pub code: &'a str,
    pub confederation: &'a str,
    pub elo_rating: f64,
    pub fifa_ranking: u16,
    pub market_value_millions: f64,
    pub world_cup_wins: u8,
}
