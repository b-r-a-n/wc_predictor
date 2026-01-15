//! Core simulation engine for running a single tournament.

use rand_chacha::ChaCha8Rng;

use wc_core::{
    Group, GroupResult, GroupStanding, KnockoutBracket, KnockoutRound, MatchResult,
    TeamId, Tournament, TournamentResult,
};
use wc_strategies::{MatchContext, PredictionStrategy};

/// Engine for simulating a single World Cup tournament.
pub struct SimulationEngine<'a> {
    tournament: &'a Tournament,
    strategy: &'a dyn PredictionStrategy,
}

impl<'a> SimulationEngine<'a> {
    /// Create a new simulation engine.
    pub fn new(tournament: &'a Tournament, strategy: &'a dyn PredictionStrategy) -> Self {
        Self {
            tournament,
            strategy,
        }
    }

    /// Run a single tournament simulation.
    pub fn simulate(&self, rng: &mut ChaCha8Rng) -> TournamentResult {
        // 1. Simulate group stage
        let group_results = self.simulate_group_stage(rng);

        // 2. Determine teams advancing to knockout
        let knockout_teams = self.determine_knockout_qualifiers(&group_results);

        // 3. Simulate knockout stage
        let knockout_bracket = self.simulate_knockout_stage(rng, knockout_teams);

        // 4. Extract final standings
        let champion = knockout_bracket.final_match.winner().unwrap();
        let runner_up = knockout_bracket.final_match.loser().unwrap();
        let third_place = knockout_bracket.third_place.winner().unwrap();
        let fourth_place = knockout_bracket.third_place.loser().unwrap();

        TournamentResult {
            group_results,
            knockout_bracket,
            champion,
            runner_up,
            third_place,
            fourth_place,
        }
    }

    /// Simulate all group stage matches.
    fn simulate_group_stage(&self, rng: &mut ChaCha8Rng) -> Vec<GroupResult> {
        self.tournament
            .groups
            .iter()
            .map(|group| self.simulate_group(group, rng))
            .collect()
    }

    /// Simulate a single group's matches.
    fn simulate_group(&self, group: &Group, rng: &mut ChaCha8Rng) -> GroupResult {
        let fixtures = group.generate_fixtures();
        let mut matches = Vec::with_capacity(6);

        for (home_id, away_id) in fixtures {
            let home_team = self.tournament.get_team(home_id).unwrap().clone();
            let away_team = self.tournament.get_team(away_id).unwrap().clone();

            let ctx = MatchContext::new(home_team, away_team, false);
            let result = self.strategy.simulate_match(&ctx, rng);
            matches.push(result);
        }

        // Calculate standings with tiebreakers
        let standings =
            wc_core::tiebreaker::calculate_standings(&group.teams, &matches);
        let standings =
            wc_core::tiebreaker::resolve_standings(standings, &matches);

        GroupResult {
            group_id: group.id,
            matches,
            standings,
        }
    }

    /// Determine the 32 teams advancing to knockout stage.
    /// Top 2 from each group (24) + 8 best third-placed teams.
    fn determine_knockout_qualifiers(&self, group_results: &[GroupResult]) -> Vec<TeamId> {
        let mut qualifiers = Vec::with_capacity(32);

        // Add group winners and runners-up (24 teams)
        for gr in group_results {
            qualifiers.push(gr.winner());
            qualifiers.push(gr.runner_up());
        }

        // Collect third-placed teams
        let third_placed: Vec<&GroupStanding> = group_results
            .iter()
            .map(|gr| &gr.standings[2])
            .collect();

        // Sort to find best 8 third-placed teams
        let third_placed = wc_core::tiebreaker::rank_third_placed_teams(
            &third_placed.iter().map(|s| (*s).clone()).collect::<Vec<_>>()
        );

        // Add best 8 third-placed teams
        for standing in third_placed.into_iter().take(8) {
            qualifiers.push(standing.team_id);
        }

        qualifiers
    }

    /// Simulate the entire knockout stage.
    fn simulate_knockout_stage(
        &self,
        rng: &mut ChaCha8Rng,
        teams: Vec<TeamId>,
    ) -> KnockoutBracket {
        // Round of 32 (16 matches)
        let round_of_32 = self.simulate_knockout_round(rng, &teams, KnockoutRound::RoundOf32);
        let ro32_winners: Vec<TeamId> = round_of_32
            .iter()
            .filter_map(|m| m.winner())
            .collect();

        // Round of 16 (8 matches)
        let round_of_16 = self.simulate_knockout_round(rng, &ro32_winners, KnockoutRound::RoundOf16);
        let ro16_winners: Vec<TeamId> = round_of_16
            .iter()
            .filter_map(|m| m.winner())
            .collect();

        // Quarter-finals (4 matches)
        let quarter_finals = self.simulate_knockout_round(rng, &ro16_winners, KnockoutRound::QuarterFinal);
        let qf_winners: Vec<TeamId> = quarter_finals
            .iter()
            .filter_map(|m| m.winner())
            .collect();

        // Semi-finals (2 matches)
        let semi_finals = self.simulate_knockout_round(rng, &qf_winners, KnockoutRound::SemiFinal);

        let sf_winners: Vec<TeamId> = semi_finals
            .iter()
            .filter_map(|m| m.winner())
            .collect();
        let sf_losers: Vec<TeamId> = semi_finals
            .iter()
            .filter_map(|m| m.loser())
            .collect();

        // Third-place match
        let third_place = self.simulate_single_match(
            rng,
            sf_losers[0],
            sf_losers[1],
            KnockoutRound::ThirdPlace,
        );

        // Final
        let final_match = self.simulate_single_match(
            rng,
            sf_winners[0],
            sf_winners[1],
            KnockoutRound::Final,
        );

        KnockoutBracket {
            round_of_32,
            round_of_16,
            quarter_finals,
            semi_finals,
            third_place,
            final_match,
        }
    }

    /// Simulate a knockout round.
    fn simulate_knockout_round(
        &self,
        rng: &mut ChaCha8Rng,
        teams: &[TeamId],
        round: KnockoutRound,
    ) -> Vec<MatchResult> {
        teams
            .chunks(2)
            .map(|pair| self.simulate_single_match(rng, pair[0], pair[1], round))
            .collect()
    }

    /// Simulate a single knockout match.
    fn simulate_single_match(
        &self,
        rng: &mut ChaCha8Rng,
        team_a: TeamId,
        team_b: TeamId,
        round: KnockoutRound,
    ) -> MatchResult {
        let home_team = self.tournament.get_team(team_a).unwrap().clone();
        let away_team = self.tournament.get_team(team_b).unwrap().clone();

        let ctx = MatchContext::new(home_team, away_team, true)
            .with_importance(round.importance());

        self.strategy.simulate_match(&ctx, rng)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use wc_core::{Confederation, Group, GroupId, Team};
    use wc_strategies::EloStrategy;

    fn create_test_tournament() -> Tournament {
        let teams: Vec<Team> = (0..48)
            .map(|i| {
                Team::new(
                    TeamId(i),
                    format!("Team {}", i),
                    format!("T{:02}", i),
                    Confederation::Uefa,
                )
                .with_elo(1800.0 - (i as f64 * 10.0)) // Vary ELO
            })
            .collect();

        let groups: Vec<Group> = (0..12)
            .map(|i| {
                let start = i * 4;
                Group::new(
                    GroupId::from_index(i as u8),
                    [
                        teams[start].id,
                        teams[start + 1].id,
                        teams[start + 2].id,
                        teams[start + 3].id,
                    ],
                )
            })
            .collect();

        Tournament::new(teams, groups)
    }

    #[test]
    fn test_full_simulation() {
        let tournament = create_test_tournament();
        let strategy = EloStrategy::default();
        let engine = SimulationEngine::new(&tournament, &strategy);

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let result = engine.simulate(&mut rng);

        // Verify structure
        assert_eq!(result.group_results.len(), 12);
        assert_eq!(result.knockout_bracket.round_of_32.len(), 16);
        assert_eq!(result.knockout_bracket.round_of_16.len(), 8);
        assert_eq!(result.knockout_bracket.quarter_finals.len(), 4);
        assert_eq!(result.knockout_bracket.semi_finals.len(), 2);

        // Verify all positions filled
        assert!(result.champion.0 < 48);
        assert!(result.runner_up.0 < 48);
        assert!(result.third_place.0 < 48);
        assert!(result.fourth_place.0 < 48);

        // All different teams in top 4
        let top4 = result.podium();
        for i in 0..4 {
            for j in (i + 1)..4 {
                assert_ne!(top4[i], top4[j]);
            }
        }
    }

    #[test]
    fn test_reproducibility() {
        let tournament = create_test_tournament();
        let strategy = EloStrategy::default();
        let engine = SimulationEngine::new(&tournament, &strategy);

        let mut rng1 = ChaCha8Rng::seed_from_u64(123);
        let result1 = engine.simulate(&mut rng1);

        let mut rng2 = ChaCha8Rng::seed_from_u64(123);
        let result2 = engine.simulate(&mut rng2);

        // Same seed should produce same results
        assert_eq!(result1.champion, result2.champion);
        assert_eq!(result1.runner_up, result2.runner_up);
    }
}
