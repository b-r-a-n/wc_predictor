//! Tournament data loading.

use std::path::Path;

use serde::Deserialize;
use wc_core::{Confederation, Group, GroupId, Team, TeamId, Tournament};

use crate::error::{CliError, Result};

/// Embedded default tournament data.
pub const EMBEDDED_DATA: &str = include_str!("../../../data/teams.json");

/// Tournament data structure matching teams.json format.
#[derive(Debug, Deserialize)]
pub struct TournamentData {
    pub teams: Vec<TeamData>,
    pub groups: Vec<GroupData>,
}

/// Team data from JSON.
#[derive(Debug, Deserialize)]
pub struct TeamData {
    pub id: u8,
    pub name: String,
    pub code: String,
    pub confederation: Confederation,
    pub elo_rating: f64,
    pub market_value_millions: f64,
    pub fifa_ranking: u16,
    pub world_cup_wins: u8,
}

/// Group data from JSON.
#[derive(Debug, Deserialize)]
pub struct GroupData {
    pub id: String,
    pub teams: [u8; 4],
}

impl TournamentData {
    /// Load from embedded data or custom file.
    pub fn load(path: Option<&Path>) -> Result<Self> {
        match path {
            Some(p) => {
                let content = std::fs::read_to_string(p).map_err(|_| {
                    CliError::InvalidDataFile(p.to_path_buf())
                })?;
                Ok(serde_json::from_str(&content)?)
            }
            None => Ok(serde_json::from_str(EMBEDDED_DATA)?),
        }
    }

    /// Convert to wc_core::Tournament.
    pub fn into_tournament(self) -> Result<Tournament> {
        let teams: Vec<Team> = self
            .teams
            .into_iter()
            .map(|t| {
                Team::new(TeamId(t.id), t.name, t.code, t.confederation)
                    .with_elo(t.elo_rating)
                    .with_market_value(t.market_value_millions)
                    .with_fifa_ranking(t.fifa_ranking)
                    .with_world_cup_wins(t.world_cup_wins)
            })
            .collect();

        let groups: Vec<Group> = self
            .groups
            .into_iter()
            .map(|g| {
                let group_id = GroupId(g.id.chars().next().unwrap_or('A'));
                Group::new(
                    group_id,
                    [
                        TeamId(g.teams[0]),
                        TeamId(g.teams[1]),
                        TeamId(g.teams[2]),
                        TeamId(g.teams[3]),
                    ],
                )
            })
            .collect();

        let tournament = Tournament::new(teams, groups);
        tournament.validate()?;
        Ok(tournament)
    }
}

/// Find a team by name or code (case-insensitive).
pub fn find_team<'a>(tournament: &'a Tournament, query: &str) -> Option<&'a Team> {
    let query_lower = query.to_lowercase();
    tournament.teams.iter().find(|t| {
        t.code.to_lowercase() == query_lower || t.name.to_lowercase() == query_lower
    })
}
