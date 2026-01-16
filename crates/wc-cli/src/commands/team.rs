//! Team info command implementation.

use wc_core::Tournament;

use crate::cli::{OutputFormat, TeamArgs};
use crate::data::find_team;
use crate::error::{CliError, Result};
use crate::output::{render_team_info_table, Output, TeamInfoJsonOutput, TeamSummary};

pub fn run_team(args: &TeamArgs, tournament: &Tournament, format: OutputFormat) -> Result<()> {
    let output = Output::new(format);

    // Find the team
    let team = find_team(tournament, &args.team)
        .ok_or_else(|| CliError::TeamNotFound(args.team.clone()))?;

    // Find the team's group
    let group = tournament.get_team_group(team.id);
    let group_id = group.map(|g| g.id.0);

    // Get group opponents
    let group_opponents: Vec<&wc_core::Team> = group
        .map(|g| {
            g.teams
                .iter()
                .filter(|&&t| t != team.id)
                .filter_map(|&t| tournament.get_team(t))
                .collect()
        })
        .unwrap_or_default();

    if output.is_json() {
        let json_output = TeamInfoJsonOutput {
            id: team.id.0,
            name: &team.name,
            code: &team.code,
            confederation: match team.confederation {
                wc_core::Confederation::Uefa => "UEFA",
                wc_core::Confederation::Conmebol => "CONMEBOL",
                wc_core::Confederation::Concacaf => "CONCACAF",
                wc_core::Confederation::Caf => "CAF",
                wc_core::Confederation::Afc => "AFC",
                wc_core::Confederation::Ofc => "OFC",
            },
            elo_rating: team.elo_rating,
            fifa_ranking: team.fifa_ranking,
            market_value_millions: team.market_value_millions,
            world_cup_wins: team.world_cup_wins,
            group: group_id,
            group_opponents: group_opponents
                .iter()
                .map(|t| TeamSummary {
                    id: t.id.0,
                    name: &t.name,
                    code: &t.code,
                })
                .collect(),
        };
        output.print_json(&json_output);
    } else {
        render_team_info_table(team, group_id, &group_opponents);
    }

    Ok(())
}
