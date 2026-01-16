//! Teams list command implementation.

use wc_core::{Confederation, Team, Tournament};

use crate::cli::{OutputFormat, TeamSortField, TeamsArgs};
use crate::error::Result;
use crate::output::{render_teams_table, Output, TeamFullInfo, TeamsListJsonOutput};

pub fn run_teams(args: &TeamsArgs, tournament: &Tournament, format: OutputFormat) -> Result<()> {
    let output = Output::new(format);

    // Filter teams by confederation if specified
    let mut teams: Vec<&Team> = tournament.teams.iter().collect();

    if let Some(conf_str) = &args.confederation {
        let conf = parse_confederation(conf_str);
        if let Some(c) = conf {
            teams.retain(|t| t.confederation == c);
        }
    }

    // Sort teams
    match args.sort {
        TeamSortField::Name => teams.sort_by(|a, b| a.name.cmp(&b.name)),
        TeamSortField::Elo => teams.sort_by(|a, b| {
            b.elo_rating
                .partial_cmp(&a.elo_rating)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        TeamSortField::FifaRank => teams.sort_by(|a, b| a.fifa_ranking.cmp(&b.fifa_ranking)),
        TeamSortField::MarketValue => teams.sort_by(|a, b| {
            b.market_value_millions
                .partial_cmp(&a.market_value_millions)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
    }

    // Reverse if requested
    if args.reverse {
        teams.reverse();
    }

    if output.is_json() {
        let json_output = TeamsListJsonOutput {
            total: teams.len(),
            teams: teams
                .iter()
                .map(|t| TeamFullInfo {
                    id: t.id.0,
                    name: &t.name,
                    code: &t.code,
                    confederation: match t.confederation {
                        Confederation::Uefa => "UEFA",
                        Confederation::Conmebol => "CONMEBOL",
                        Confederation::Concacaf => "CONCACAF",
                        Confederation::Caf => "CAF",
                        Confederation::Afc => "AFC",
                        Confederation::Ofc => "OFC",
                    },
                    elo_rating: t.elo_rating,
                    fifa_ranking: t.fifa_ranking,
                    market_value_millions: t.market_value_millions,
                    world_cup_wins: t.world_cup_wins,
                })
                .collect(),
        };
        output.print_json(&json_output);
    } else {
        render_teams_table(&teams);
    }

    Ok(())
}

fn parse_confederation(s: &str) -> Option<Confederation> {
    match s.to_uppercase().as_str() {
        "UEFA" => Some(Confederation::Uefa),
        "CONMEBOL" => Some(Confederation::Conmebol),
        "CONCACAF" => Some(Confederation::Concacaf),
        "CAF" => Some(Confederation::Caf),
        "AFC" => Some(Confederation::Afc),
        "OFC" => Some(Confederation::Ofc),
        _ => None,
    }
}
