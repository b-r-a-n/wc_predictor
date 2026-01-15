//! Team and confederation types.

use serde::{Deserialize, Serialize};

/// Unique identifier for a team (0-47 for 48 teams).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(transparent)]
pub struct TeamId(pub u8);

impl Default for TeamId {
    fn default() -> Self {
        Self(0)
    }
}

/// FIFA confederation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Confederation {
    /// Union of European Football Associations
    Uefa,
    /// Confederación Sudamericana de Fútbol
    Conmebol,
    /// Confederation of North, Central America and Caribbean Association Football
    Concacaf,
    /// Confédération Africaine de Football
    Caf,
    /// Asian Football Confederation
    Afc,
    /// Oceania Football Confederation
    Ofc,
}

/// A national team with all relevant statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    /// Unique identifier
    pub id: TeamId,
    /// Full team name (e.g., "Brazil")
    pub name: String,
    /// FIFA country code (e.g., "BRA")
    pub code: String,
    /// FIFA confederation
    pub confederation: Confederation,
    /// World Football ELO rating
    pub elo_rating: f64,
    /// Squad market value in millions of euros
    pub market_value_millions: f64,
    /// FIFA world ranking position (1-211)
    pub fifa_ranking: u16,
    /// Number of World Cup titles won
    pub world_cup_wins: u8,
}

impl Team {
    /// Create a new team with the given parameters.
    pub fn new(
        id: TeamId,
        name: impl Into<String>,
        code: impl Into<String>,
        confederation: Confederation,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            code: code.into(),
            confederation,
            elo_rating: 1500.0,
            market_value_millions: 0.0,
            fifa_ranking: 100,
            world_cup_wins: 0,
        }
    }

    /// Builder method to set ELO rating.
    pub fn with_elo(mut self, elo: f64) -> Self {
        self.elo_rating = elo;
        self
    }

    /// Builder method to set market value.
    pub fn with_market_value(mut self, value: f64) -> Self {
        self.market_value_millions = value;
        self
    }

    /// Builder method to set FIFA ranking.
    pub fn with_fifa_ranking(mut self, ranking: u16) -> Self {
        self.fifa_ranking = ranking;
        self
    }

    /// Builder method to set World Cup wins.
    pub fn with_world_cup_wins(mut self, wins: u8) -> Self {
        self.world_cup_wins = wins;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_builder() {
        let team = Team::new(TeamId(0), "Brazil", "BRA", Confederation::Conmebol)
            .with_elo(2100.0)
            .with_market_value(1200.0)
            .with_fifa_ranking(1)
            .with_world_cup_wins(5);

        assert_eq!(team.name, "Brazil");
        assert_eq!(team.elo_rating, 2100.0);
        assert_eq!(team.world_cup_wins, 5);
    }
}
