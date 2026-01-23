//! Core simulation engine for running a single tournament.

use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;

use wc_core::{
    bracket, FixedResults, Group, GroupResult, GroupStanding, KnockoutBracket, KnockoutRound,
    MatchResult, TeamId, Tournament, TournamentResult,
};
use wc_strategies::{MatchContext, PredictionStrategy};

/// Engine for simulating a single World Cup tournament.
pub struct SimulationEngine<'a> {
    tournament: &'a Tournament,
    strategy: &'a dyn PredictionStrategy,
    fixed_results: Option<&'a FixedResults>,
}

impl<'a> SimulationEngine<'a> {
    /// Create a new simulation engine.
    pub fn new(tournament: &'a Tournament, strategy: &'a dyn PredictionStrategy) -> Self {
        Self {
            tournament,
            strategy,
            fixed_results: None,
        }
    }

    /// Add fixed results to the engine (builder pattern).
    pub fn with_fixed_results(mut self, fixed: &'a FixedResults) -> Self {
        self.fixed_results = Some(fixed);
        self
    }

    /// Run a single tournament simulation.
    pub fn simulate(&self, rng: &mut ChaCha8Rng) -> TournamentResult {
        // 1. Simulate group stage
        let group_results = self.simulate_group_stage(rng);

        // 2. Build R32 pairings using official FIFA 2026 bracket structure
        let r32_pairings = self.build_r32_pairings(&group_results);

        // 3. Simulate knockout stage
        let knockout_bracket = self.simulate_knockout_stage(rng, r32_pairings);

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
            // Check if this match has a fixed result
            let result = if let Some(fixed) = self.fixed_results {
                if let Some(spec) = fixed.get_group_match(group.id, home_id, away_id) {
                    // Use the fixed result
                    spec.to_match_result(home_id, away_id, false, rng)
                } else {
                    // Simulate normally
                    self.simulate_group_match(rng, home_id, away_id)
                }
            } else {
                // No fixed results, simulate normally
                self.simulate_group_match(rng, home_id, away_id)
            };

            matches.push(result);
        }

        // Calculate standings with tiebreakers
        let standings =
            wc_core::tiebreaker::calculate_standings(&group.teams, &matches, group.id);
        let standings = wc_core::tiebreaker::resolve_standings(standings, &matches);

        GroupResult {
            group_id: group.id,
            matches,
            standings,
        }
    }

    /// Simulate a single group stage match using the strategy.
    fn simulate_group_match(
        &self,
        rng: &mut ChaCha8Rng,
        home_id: TeamId,
        away_id: TeamId,
    ) -> MatchResult {
        let home_team = self.tournament.get_team(home_id).unwrap().clone();
        let away_team = self.tournament.get_team(away_id).unwrap().clone();

        let ctx = MatchContext::new(home_team, away_team, false);
        self.strategy.simulate_match(&ctx, rng)
    }

    /// Build Round of 32 pairings using official FIFA 2026 bracket structure.
    ///
    /// Returns 16 pairs of TeamIds in bracket order (ordered so .chunks(2) feeds
    /// correctly into Round of 16).
    fn build_r32_pairings(&self, group_results: &[GroupResult]) -> Vec<(TeamId, TeamId)> {
        // Build lookup maps for winners and runners-up
        let mut winners: HashMap<char, TeamId> = HashMap::new();
        let mut runners_up: HashMap<char, TeamId> = HashMap::new();

        for gr in group_results {
            let group_char = gr.group_id.0;
            winners.insert(group_char, gr.winner());
            runners_up.insert(group_char, gr.runner_up());
        }

        // Collect and rank third-placed teams
        let third_standings: Vec<GroupStanding> = group_results
            .iter()
            .map(|gr| gr.standings[2].clone())
            .collect();

        let ranked_thirds = wc_core::tiebreaker::rank_third_placed_teams(&third_standings);

        // Take best 8 third-placed teams with their group letters
        let qualifying_thirds: Vec<(char, TeamId)> = ranked_thirds
            .into_iter()
            .take(8)
            .map(|s| (s.group_id.0, s.team_id))
            .collect();

        // Use bracket module to build pairings
        bracket::build_r32_pairings(&winners, &runners_up, &qualifying_thirds)
    }

    /// Simulate the entire knockout stage.
    fn simulate_knockout_stage(
        &self,
        rng: &mut ChaCha8Rng,
        r32_pairings: Vec<(TeamId, TeamId)>,
    ) -> KnockoutBracket {
        // Round of 32 (16 matches) - use explicit pairings from bracket module
        let round_of_32: Vec<MatchResult> = r32_pairings
            .iter()
            .enumerate()
            .map(|(slot, (team_a, team_b))| {
                self.simulate_knockout_match_with_slot(
                    rng,
                    *team_a,
                    *team_b,
                    KnockoutRound::RoundOf32,
                    slot as u8,
                )
            })
            .collect();
        let ro32_winners: Vec<TeamId> = round_of_32.iter().filter_map(|m| m.winner()).collect();

        // Round of 16 (8 matches)
        let round_of_16 =
            self.simulate_knockout_round_with_slots(rng, &ro32_winners, KnockoutRound::RoundOf16);
        let ro16_winners: Vec<TeamId> = round_of_16.iter().filter_map(|m| m.winner()).collect();

        // Quarter-finals (4 matches)
        let quarter_finals =
            self.simulate_knockout_round_with_slots(rng, &ro16_winners, KnockoutRound::QuarterFinal);
        let qf_winners: Vec<TeamId> = quarter_finals.iter().filter_map(|m| m.winner()).collect();

        // Semi-finals (2 matches)
        let semi_finals =
            self.simulate_knockout_round_with_slots(rng, &qf_winners, KnockoutRound::SemiFinal);

        let sf_winners: Vec<TeamId> = semi_finals.iter().filter_map(|m| m.winner()).collect();
        let sf_losers: Vec<TeamId> = semi_finals.iter().filter_map(|m| m.loser()).collect();

        // Third-place match
        let third_place = self.simulate_knockout_match_with_slot(
            rng,
            sf_losers[0],
            sf_losers[1],
            KnockoutRound::ThirdPlace,
            0,
        );

        // Final
        let final_match = self.simulate_knockout_match_with_slot(
            rng,
            sf_winners[0],
            sf_winners[1],
            KnockoutRound::Final,
            0,
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

    /// Simulate a knockout round with slot tracking for fixed results.
    fn simulate_knockout_round_with_slots(
        &self,
        rng: &mut ChaCha8Rng,
        teams: &[TeamId],
        round: KnockoutRound,
    ) -> Vec<MatchResult> {
        teams
            .chunks(2)
            .enumerate()
            .map(|(slot, pair)| {
                self.simulate_knockout_match_with_slot(rng, pair[0], pair[1], round, slot as u8)
            })
            .collect()
    }

    /// Simulate a single knockout match, checking for fixed results first.
    fn simulate_knockout_match_with_slot(
        &self,
        rng: &mut ChaCha8Rng,
        team_a: TeamId,
        team_b: TeamId,
        round: KnockoutRound,
        slot: u8,
    ) -> MatchResult {
        // Check if this match has a fixed result
        if let Some(fixed) = self.fixed_results {
            if let Some(spec) = fixed.get_knockout_match(round, slot) {
                return spec.to_match_result(team_a, team_b, true, rng);
            }
        }

        // No fixed result, simulate normally
        let home_team = self.tournament.get_team(team_a).unwrap().clone();
        let away_team = self.tournament.get_team(team_b).unwrap().clone();

        let ctx = MatchContext::new(home_team, away_team, true).with_importance(round.importance());

        self.strategy.simulate_match(&ctx, rng)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use wc_core::{Confederation, FixedResultSpec, Group, GroupId, MatchFixture, Team};
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

    #[test]
    fn test_winner_group_a_plays_third_place() {
        // Verify that Winner of Group A plays a third-place team, NOT Runner-up A
        // According to FIFA 2026 bracket: Match 79 is 1A vs 3(C/E/F/H/I)
        let tournament = create_test_tournament();
        let strategy = EloStrategy::default();
        let engine = SimulationEngine::new(&tournament, &strategy);

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let result = engine.simulate(&mut rng);

        // Get Group A winner and runner-up
        let group_a = &result.group_results[0];
        assert_eq!(group_a.group_id.0, 'A');
        let winner_a = group_a.winner();
        let runner_up_a = group_a.runner_up();

        // Find Match 79 in R32 (slot 6 in our bracket ordering)
        // According to R32_BRACKET, slot 6 is Match 79: 1A vs 3(C/E/F/H/I)
        let match_79 = &result.knockout_bracket.round_of_32[6];

        // One of the teams in match 79 should be Winner A
        let has_winner_a = match_79.home_team == winner_a || match_79.away_team == winner_a;
        assert!(has_winner_a, "Match 79 should include Winner of Group A");

        // Neither team should be Runner-up A (the old incorrect pairing)
        let has_runner_up_a = match_79.home_team == runner_up_a || match_79.away_team == runner_up_a;
        assert!(!has_runner_up_a, "Match 79 should NOT include Runner-up of Group A");
    }

    // ==================== FIXED RESULTS TESTS ====================

    #[test]
    fn test_fixed_single_group_match() {
        let tournament = create_test_tournament();
        let strategy = EloStrategy::default();

        // Fix first match of Group A: Team 0 vs Team 1 → Team 0 wins 3-0
        let mut fixed = FixedResults::new();
        fixed.insert(
            MatchFixture::group_stage(GroupId('A'), TeamId(0), TeamId(1)),
            FixedResultSpec::exact_score(3, 0),
        );

        let engine = SimulationEngine::new(&tournament, &strategy).with_fixed_results(&fixed);

        // Run multiple simulations - fixed result should appear every time
        for seed in 0..10 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let result = engine.simulate(&mut rng);

            let group_a = &result.group_results[0];
            let first_match = &group_a.matches[0];

            assert_eq!(first_match.home_team, TeamId(0));
            assert_eq!(first_match.away_team, TeamId(1));
            assert_eq!(first_match.home_goals, 3, "Fixed score should be 3-0");
            assert_eq!(first_match.away_goals, 0, "Fixed score should be 3-0");
        }
    }

    #[test]
    fn test_fixed_multiple_group_matches_same_group() {
        let tournament = create_test_tournament();
        let strategy = EloStrategy::default();

        // Fix multiple matches in Group A
        let mut fixed = FixedResults::new();
        fixed.insert(
            MatchFixture::group_stage(GroupId('A'), TeamId(0), TeamId(1)),
            FixedResultSpec::exact_score(2, 0),
        );
        fixed.insert(
            MatchFixture::group_stage(GroupId('A'), TeamId(2), TeamId(3)),
            FixedResultSpec::exact_score(1, 1),
        );

        let engine = SimulationEngine::new(&tournament, &strategy).with_fixed_results(&fixed);

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let result = engine.simulate(&mut rng);

        let group_a = &result.group_results[0];

        // First match: 0 vs 1 → 2-0
        assert_eq!(group_a.matches[0].home_goals, 2);
        assert_eq!(group_a.matches[0].away_goals, 0);

        // Second match: 2 vs 3 → 1-1
        assert_eq!(group_a.matches[1].home_goals, 1);
        assert_eq!(group_a.matches[1].away_goals, 1);
    }

    #[test]
    fn test_fixed_r32_match() {
        let tournament = create_test_tournament();
        let strategy = EloStrategy::default();

        // Fix R32 slot 0 to have exact score 2-1
        let mut fixed = FixedResults::new();
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::RoundOf32, 0),
            FixedResultSpec::exact_score(2, 1),
        );

        let engine = SimulationEngine::new(&tournament, &strategy).with_fixed_results(&fixed);

        for seed in 0..5 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let result = engine.simulate(&mut rng);

            let r32_match_0 = &result.knockout_bracket.round_of_32[0];
            assert_eq!(r32_match_0.home_goals, 2, "R32 slot 0 should be 2-1");
            assert_eq!(r32_match_0.away_goals, 1, "R32 slot 0 should be 2-1");
        }
    }

    #[test]
    fn test_fixed_final_exact_score() {
        let tournament = create_test_tournament();
        let strategy = EloStrategy::default();

        // Just fix the final score - doesn't matter who plays, the score should be fixed
        let mut fixed = FixedResults::new();
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::Final, 0),
            FixedResultSpec::exact_score(3, 2),
        );

        let engine = SimulationEngine::new(&tournament, &strategy).with_fixed_results(&fixed);

        // Run multiple simulations - final should always be 3-2
        for seed in 0..10 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let result = engine.simulate(&mut rng);

            let final_match = &result.knockout_bracket.final_match;
            assert_eq!(
                final_match.home_goals, 3,
                "Final should always be 3-2 (seed {})",
                seed
            );
            assert_eq!(
                final_match.away_goals, 2,
                "Final should always be 3-2 (seed {})",
                seed
            );
            // Home team should always win
            assert_eq!(
                final_match.winner(),
                Some(final_match.home_team),
                "Home team should win 3-2"
            );
        }
    }

    #[test]
    fn test_fixed_semifinal_determines_finalists() {
        let tournament = create_test_tournament();
        let strategy = EloStrategy::default();

        // Fix both semifinals with exact scores
        let mut fixed = FixedResults::new();
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::SemiFinal, 0),
            FixedResultSpec::exact_score(2, 0), // Home wins
        );
        fixed.insert(
            MatchFixture::knockout(KnockoutRound::SemiFinal, 1),
            FixedResultSpec::exact_score(0, 1), // Away wins
        );

        let engine = SimulationEngine::new(&tournament, &strategy).with_fixed_results(&fixed);

        for seed in 0..5 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let result = engine.simulate(&mut rng);

            // SF 0 should be 2-0
            let sf0 = &result.knockout_bracket.semi_finals[0];
            assert_eq!(sf0.home_goals, 2);
            assert_eq!(sf0.away_goals, 0);

            // SF 1 should be 0-1
            let sf1 = &result.knockout_bracket.semi_finals[1];
            assert_eq!(sf1.home_goals, 0);
            assert_eq!(sf1.away_goals, 1);

            // Final should have: SF0 home team vs SF1 away team
            let final_match = &result.knockout_bracket.final_match;
            assert_eq!(final_match.home_team, sf0.home_team);
            assert_eq!(final_match.away_team, sf1.away_team);
        }
    }

    #[test]
    fn test_winner_only_produces_valid_score() {
        let tournament = create_test_tournament();
        let strategy = EloStrategy::default();

        let mut fixed = FixedResults::new();
        fixed.insert(
            MatchFixture::group_stage(GroupId('A'), TeamId(0), TeamId(1)),
            FixedResultSpec::winner_only(TeamId(0)),
        );

        let engine = SimulationEngine::new(&tournament, &strategy).with_fixed_results(&fixed);

        for seed in 0..20 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let result = engine.simulate(&mut rng);

            let group_a = &result.group_results[0];
            let first_match = &group_a.matches[0];

            assert_eq!(
                first_match.winner(),
                Some(TeamId(0)),
                "Team 0 should always win"
            );
            assert!(
                first_match.home_goals > first_match.away_goals,
                "Home team (0) should have more goals"
            );
        }
    }

    #[test]
    fn test_fixed_results_deterministic() {
        let tournament = create_test_tournament();
        let strategy = EloStrategy::default();

        let mut fixed = FixedResults::new();
        // Use winner_only which uses RNG
        fixed.insert(
            MatchFixture::group_stage(GroupId('A'), TeamId(0), TeamId(1)),
            FixedResultSpec::winner_only(TeamId(0)),
        );

        let engine = SimulationEngine::new(&tournament, &strategy).with_fixed_results(&fixed);

        // Same seed should produce same result
        let mut rng1 = ChaCha8Rng::seed_from_u64(12345);
        let result1 = engine.simulate(&mut rng1);

        let mut rng2 = ChaCha8Rng::seed_from_u64(12345);
        let result2 = engine.simulate(&mut rng2);

        let match1 = &result1.group_results[0].matches[0];
        let match2 = &result2.group_results[0].matches[0];

        assert_eq!(match1.home_goals, match2.home_goals);
        assert_eq!(match1.away_goals, match2.away_goals);
        assert_eq!(result1.champion, result2.champion);
    }

    #[test]
    fn test_mix_fixed_and_simulated() {
        let tournament = create_test_tournament();
        let strategy = EloStrategy::default();

        // Fix only one match, rest should be simulated normally
        let mut fixed = FixedResults::new();
        fixed.insert(
            MatchFixture::group_stage(GroupId('A'), TeamId(0), TeamId(1)),
            FixedResultSpec::exact_score(5, 0),
        );

        let engine = SimulationEngine::new(&tournament, &strategy).with_fixed_results(&fixed);

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let result = engine.simulate(&mut rng);

        // Fixed match should have exact score
        let group_a = &result.group_results[0];
        assert_eq!(group_a.matches[0].home_goals, 5);
        assert_eq!(group_a.matches[0].away_goals, 0);

        // Other matches in group A should be simulated (not all 5-0)
        let other_matches_varied = group_a.matches[1..].iter().any(|m| {
            m.home_goals != 5 || m.away_goals != 0
        });
        assert!(other_matches_varied, "Non-fixed matches should be simulated normally");

        // Other groups should be simulated normally
        let group_b = &result.group_results[1];
        let group_b_has_varied = group_b.matches.iter().any(|m| {
            m.home_goals != 5 || m.away_goals != 0
        });
        assert!(group_b_has_varied, "Group B should have varied scores");
    }

    #[test]
    fn test_no_fixed_results_unchanged_behavior() {
        let tournament = create_test_tournament();
        let strategy = EloStrategy::default();

        // Engine without fixed results
        let engine_no_fixed = SimulationEngine::new(&tournament, &strategy);

        // Engine with empty fixed results
        let fixed = FixedResults::new();
        let engine_empty_fixed =
            SimulationEngine::new(&tournament, &strategy).with_fixed_results(&fixed);

        let mut rng1 = ChaCha8Rng::seed_from_u64(42);
        let result1 = engine_no_fixed.simulate(&mut rng1);

        let mut rng2 = ChaCha8Rng::seed_from_u64(42);
        let result2 = engine_empty_fixed.simulate(&mut rng2);

        // Should produce identical results
        assert_eq!(result1.champion, result2.champion);
        assert_eq!(result1.runner_up, result2.runner_up);
    }
}
