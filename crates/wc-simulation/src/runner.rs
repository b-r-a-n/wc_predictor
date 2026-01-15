//! Monte Carlo simulation runner for parallel execution.

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;

use wc_core::{Tournament, TournamentResult};
use wc_strategies::PredictionStrategy;

use crate::aggregator::AggregatedResults;
use crate::engine::SimulationEngine;

/// Configuration for Monte Carlo simulation.
#[derive(Debug, Clone)]
pub struct SimulationConfig {
    /// Number of tournament simulations to run
    pub iterations: u32,
    /// Base seed for RNG (None for random)
    pub seed: Option<u64>,
    /// Number of parallel threads (None for auto-detect)
    pub parallelism: Option<usize>,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            iterations: 10_000,
            seed: None,
            parallelism: None,
        }
    }
}

impl SimulationConfig {
    /// Create a new config with the specified iterations.
    pub fn with_iterations(iterations: u32) -> Self {
        Self {
            iterations,
            ..Default::default()
        }
    }

    /// Set the seed for reproducibility.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Set the number of parallel threads.
    pub fn with_parallelism(mut self, threads: usize) -> Self {
        self.parallelism = Some(threads);
        self
    }
}

/// Monte Carlo simulation runner.
pub struct SimulationRunner<'a> {
    tournament: &'a Tournament,
    strategy: &'a dyn PredictionStrategy,
    config: SimulationConfig,
}

impl<'a> SimulationRunner<'a> {
    /// Create a new simulation runner.
    pub fn new(
        tournament: &'a Tournament,
        strategy: &'a dyn PredictionStrategy,
        config: SimulationConfig,
    ) -> Self {
        Self {
            tournament,
            strategy,
            config,
        }
    }

    /// Run simulations in parallel and return aggregated results.
    pub fn run(&self) -> AggregatedResults {
        let base_seed = self.config.seed.unwrap_or_else(|| {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64
        });

        // Configure thread pool if specified
        if let Some(threads) = self.config.parallelism {
            rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build_global()
                .ok();
        }

        // Run simulations in parallel
        let results: Vec<TournamentResult> = (0..self.config.iterations)
            .into_par_iter()
            .map(|i| {
                let seed = base_seed.wrapping_add(i as u64);
                let mut rng = ChaCha8Rng::seed_from_u64(seed);

                let engine = SimulationEngine::new(self.tournament, self.strategy);
                engine.simulate(&mut rng)
            })
            .collect();

        AggregatedResults::from_results(results, self.tournament)
    }

    /// Run simulations sequentially with progress callback.
    /// Useful for single-threaded environments (WASM).
    pub fn run_with_progress<F>(&self, mut on_progress: F) -> AggregatedResults
    where
        F: FnMut(u32, u32),
    {
        let base_seed = self.config.seed.unwrap_or(42);
        let mut results = Vec::with_capacity(self.config.iterations as usize);

        for i in 0..self.config.iterations {
            let seed = base_seed.wrapping_add(i as u64);
            let mut rng = ChaCha8Rng::seed_from_u64(seed);

            let engine = SimulationEngine::new(self.tournament, self.strategy);
            results.push(engine.simulate(&mut rng));

            // Report progress periodically
            if i % 100 == 0 || i == self.config.iterations - 1 {
                on_progress(i + 1, self.config.iterations);
            }
        }

        AggregatedResults::from_results(results, self.tournament)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wc_core::{Confederation, Group, GroupId, Team, TeamId};
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
                .with_elo(1800.0 - (i as f64 * 10.0))
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
    fn test_runner_basic() {
        let tournament = create_test_tournament();
        let strategy = EloStrategy::default();
        let config = SimulationConfig::with_iterations(100).with_seed(42);

        let runner = SimulationRunner::new(&tournament, &strategy, config);
        let results = runner.run();

        assert_eq!(results.total_simulations, 100);
        assert_eq!(results.team_stats.len(), 48);
    }

    #[test]
    fn test_reproducibility_with_seed() {
        let tournament = create_test_tournament();
        let strategy = EloStrategy::default();

        // Use run_with_progress for deterministic sequential execution
        let config = SimulationConfig::with_iterations(100).with_seed(999);
        let runner = SimulationRunner::new(&tournament, &strategy, config.clone());
        let results1 = runner.run_with_progress(|_, _| {});

        let runner = SimulationRunner::new(&tournament, &strategy, config);
        let results2 = runner.run_with_progress(|_, _| {});

        // Verify exact same results by comparing champion counts for all teams
        for (team_id, stats1) in &results1.team_stats {
            let stats2 = results2.team_stats.get(team_id).unwrap();
            assert_eq!(stats1.champion, stats2.champion, "Champion counts differ for {:?}", team_id);
        }
    }

    #[test]
    fn test_progress_callback() {
        let tournament = create_test_tournament();
        let strategy = EloStrategy::default();
        let config = SimulationConfig::with_iterations(200).with_seed(42);

        let runner = SimulationRunner::new(&tournament, &strategy, config);

        let mut progress_calls = 0;
        let results = runner.run_with_progress(|completed, total| {
            progress_calls += 1;
            assert!(completed <= total);
        });

        assert!(progress_calls > 0);
        assert_eq!(results.total_simulations, 200);
    }
}
