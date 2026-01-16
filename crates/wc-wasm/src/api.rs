//! WASM API for World Cup simulation.

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};

use wc_core::{TeamId, Tournament};
use wc_simulation::{AggregatedResults, SimulationConfig, SimulationRunner};
use wc_strategies::{
    CompositeStrategy, EloStrategy, FifaRankingStrategy, MarketValueStrategy, PredictionStrategy,
};

/// Main simulator interface for JavaScript.
#[wasm_bindgen]
pub struct WcSimulator {
    tournament: Tournament,
}

#[wasm_bindgen]
impl WcSimulator {
    /// Create a new simulator from tournament JSON data.
    ///
    /// # Arguments
    /// * `tournament_json` - JSON object containing teams and groups
    ///
    /// # Example
    /// ```javascript
    /// const simulator = new WcSimulator({
    ///   teams: [...],
    ///   groups: [...]
    /// });
    /// ```
    #[wasm_bindgen(constructor)]
    pub fn new(tournament_json: JsValue) -> Result<WcSimulator, JsError> {
        let tournament: Tournament = from_value(tournament_json)
            .map_err(|e| JsError::new(&format!("Invalid tournament data: {}", e)))?;

        // Validate tournament structure
        tournament
            .validate()
            .map_err(|e| JsError::new(&format!("Invalid tournament: {}", e)))?;

        Ok(Self { tournament })
    }

    /// Run simulation using ELO-based predictions.
    ///
    /// # Arguments
    /// * `iterations` - Number of tournament simulations to run
    /// * `seed` - Optional seed for reproducibility
    #[wasm_bindgen(js_name = runEloSimulation)]
    pub fn run_elo_simulation(
        &self,
        iterations: u32,
        seed: Option<u64>,
    ) -> Result<JsValue, JsError> {
        let strategy = EloStrategy::default();
        self.run_simulation(&strategy, iterations, seed)
    }

    /// Run simulation using market value-based predictions.
    #[wasm_bindgen(js_name = runMarketValueSimulation)]
    pub fn run_market_value_simulation(
        &self,
        iterations: u32,
        seed: Option<u64>,
    ) -> Result<JsValue, JsError> {
        let strategy = MarketValueStrategy::default();
        self.run_simulation(&strategy, iterations, seed)
    }

    /// Run simulation using FIFA ranking-based predictions.
    #[wasm_bindgen(js_name = runFifaRankingSimulation)]
    pub fn run_fifa_ranking_simulation(
        &self,
        iterations: u32,
        seed: Option<u64>,
    ) -> Result<JsValue, JsError> {
        let strategy = FifaRankingStrategy::default();
        self.run_simulation(&strategy, iterations, seed)
    }

    /// Run simulation using a composite strategy.
    ///
    /// # Arguments
    /// * `elo_weight` - Weight for ELO strategy (0.0 to 1.0)
    /// * `market_weight` - Weight for market value strategy
    /// * `fifa_weight` - Weight for FIFA ranking strategy
    /// * `iterations` - Number of simulations
    /// * `seed` - Optional seed
    #[wasm_bindgen(js_name = runCompositeSimulation)]
    pub fn run_composite_simulation(
        &self,
        elo_weight: f64,
        market_weight: f64,
        fifa_weight: f64,
        iterations: u32,
        seed: Option<u64>,
    ) -> Result<JsValue, JsError> {
        let strategy = CompositeStrategy::new("Composite")
            .add_strategy(EloStrategy::default(), elo_weight)
            .add_strategy(MarketValueStrategy::default(), market_weight)
            .add_strategy(FifaRankingStrategy::default(), fifa_weight);

        self.run_simulation(&strategy, iterations, seed)
    }

    /// Internal simulation runner.
    fn run_simulation(
        &self,
        strategy: &dyn PredictionStrategy,
        iterations: u32,
        seed: Option<u64>,
    ) -> Result<JsValue, JsError> {
        let mut config = SimulationConfig::with_iterations(iterations);
        config.parallelism = Some(1); // Single-threaded in WASM

        if let Some(s) = seed {
            config = config.with_seed(s);
        }

        let runner = SimulationRunner::new(&self.tournament, strategy, config);
        let results = runner.run_with_progress(|_, _| {});

        // Workaround: serde_wasm_bindgen drops some fields, so serialize to JSON and parse
        let json_str = serde_json::to_string(&results)
            .map_err(|e| JsError::new(&format!("JSON serialization error: {}", e)))?;

        let js_value = js_sys::JSON::parse(&json_str)
            .map_err(|e| JsError::new(&format!("JSON parse error: {:?}", e)))?;

        Ok(js_value)
    }

    /// Get the list of teams.
    #[wasm_bindgen(js_name = getTeams)]
    pub fn get_teams(&self) -> Result<JsValue, JsError> {
        to_value(&self.tournament.teams)
            .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
    }

    /// Get the group configuration.
    #[wasm_bindgen(js_name = getGroups)]
    pub fn get_groups(&self) -> Result<JsValue, JsError> {
        to_value(&self.tournament.groups)
            .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
    }

    /// Get the number of teams.
    #[wasm_bindgen(js_name = numTeams)]
    pub fn num_teams(&self) -> usize {
        self.tournament.teams.len()
    }

    /// Get the number of groups.
    #[wasm_bindgen(js_name = numGroups)]
    pub fn num_groups(&self) -> usize {
        self.tournament.groups.len()
    }
}

/// Run a single tournament simulation and return detailed results.
///
/// Useful for step-by-step visualization of a single tournament.
#[wasm_bindgen(js_name = simulateSingleTournament)]
pub fn simulate_single_tournament(
    tournament_json: JsValue,
    strategy: &str,
    seed: u64,
) -> Result<JsValue, JsError> {
    let tournament: Tournament = from_value(tournament_json)
        .map_err(|e| JsError::new(&format!("Invalid tournament data: {}", e)))?;

    tournament
        .validate()
        .map_err(|e| JsError::new(&format!("Invalid tournament: {}", e)))?;

    let strategy_impl: Box<dyn PredictionStrategy> = match strategy {
        "elo" => Box::new(EloStrategy::default()),
        "market_value" => Box::new(MarketValueStrategy::default()),
        "fifa_ranking" => Box::new(FifaRankingStrategy::default()),
        "composite" => Box::new(
            CompositeStrategy::new("Default Composite")
                .add_strategy(EloStrategy::default(), 0.4)
                .add_strategy(MarketValueStrategy::default(), 0.3)
                .add_strategy(FifaRankingStrategy::default(), 0.3),
        ),
        _ => return Err(JsError::new(&format!("Unknown strategy: {}", strategy))),
    };

    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use wc_simulation::SimulationEngine;

    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let engine = SimulationEngine::new(&tournament, strategy_impl.as_ref());
    let result = engine.simulate(&mut rng);

    to_value(&result).map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
}

/// Calculate head-to-head match probability between two teams.
///
/// Returns an object with home_win, draw, and away_win probabilities.
#[wasm_bindgen(js_name = calculateMatchProbability)]
pub fn calculate_match_probability(
    team_a_elo: f64,
    team_b_elo: f64,
    is_knockout: bool,
) -> JsValue {
    use wc_core::{Confederation, Team, TeamId};
    use wc_strategies::MatchContext;

    let team_a = Team::new(TeamId(0), "Team A", "TA", Confederation::Uefa).with_elo(team_a_elo);
    let team_b = Team::new(TeamId(1), "Team B", "TB", Confederation::Uefa).with_elo(team_b_elo);

    let ctx = MatchContext::new(team_a, team_b, is_knockout);
    let strategy = EloStrategy::default();
    let probs = strategy.predict_probabilities(&ctx);

    to_value(&probs).unwrap()
}

/// Get version information.
#[wasm_bindgen(js_name = getVersion)]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// A single tournament path entry with human-readable information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathEntry {
    /// The path string (e.g., "R32:5,R16:12,QF:3,SF:14,F:0")
    pub path: String,
    /// Number of times this path occurred
    pub count: u32,
    /// Probability of this path (count / total_simulations)
    pub probability: f64,
}

/// Result of getting top paths for a team.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopPathsResult {
    /// Team ID
    pub team_id: u8,
    /// Total simulations run
    pub total_simulations: u32,
    /// Whether this team has path statistics (false if eliminated in group stage always)
    pub has_paths: bool,
    /// The top N paths sorted by count descending
    pub paths: Vec<PathEntry>,
}

/// Get top N tournament paths for a specific team.
///
/// This helper function extracts and formats path statistics from simulation results.
/// Useful for visualizing the most common tournament paths a team takes.
///
/// # Arguments
/// * `results` - The AggregatedResults from a simulation run
/// * `team_id` - The team ID to get paths for
/// * `top_n` - Maximum number of paths to return
///
/// # Returns
/// A TopPathsResult containing the top N paths sorted by occurrence count.
///
/// # Example
/// ```javascript
/// const results = simulator.runEloSimulation(10000, undefined);
/// const topPaths = getTopPathsForTeam(results, 0, 10);
/// console.log(topPaths.paths);
/// ```
#[wasm_bindgen(js_name = getTopPathsForTeam)]
pub fn get_top_paths_for_team(
    results: JsValue,
    team_id: u8,
    top_n: usize,
) -> Result<JsValue, JsError> {
    let aggregated: AggregatedResults = from_value(results)
        .map_err(|e| JsError::new(&format!("Failed to deserialize results: {}", e)))?;

    let team = TeamId(team_id);
    let total_simulations = aggregated.total_simulations;

    // Check if team has path statistics
    let path_stats = aggregated.path_stats.get(&team);

    let result = match path_stats {
        Some(stats) => {
            // Get all paths and sort by count descending
            let mut paths: Vec<_> = stats.complete_paths.iter().collect();
            paths.sort_by(|a, b| b.1.cmp(a.1));

            // Take top N and convert to PathEntry
            let top_paths: Vec<PathEntry> = paths
                .into_iter()
                .take(top_n)
                .map(|(path, count)| PathEntry {
                    path: path.clone(),
                    count: *count,
                    probability: *count as f64 / total_simulations as f64,
                })
                .collect();

            TopPathsResult {
                team_id,
                total_simulations,
                has_paths: !top_paths.is_empty(),
                paths: top_paths,
            }
        }
        None => TopPathsResult {
            team_id,
            total_simulations,
            has_paths: false,
            paths: vec![],
        },
    };

    to_value(&result).map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
}

/// Get matchup frequencies for a specific team at each knockout round.
///
/// Returns opponent frequencies at each round of the knockout stage.
///
/// # Arguments
/// * `results` - The AggregatedResults from a simulation run
/// * `team_id` - The team ID to get matchup stats for
///
/// # Returns
/// An object containing matchup frequencies for each knockout round.
#[wasm_bindgen(js_name = getTeamMatchupStats)]
pub fn get_team_matchup_stats(results: JsValue, team_id: u8) -> Result<JsValue, JsError> {
    let aggregated: AggregatedResults = from_value(results)
        .map_err(|e| JsError::new(&format!("Failed to deserialize results: {}", e)))?;

    let team = TeamId(team_id);

    match aggregated.path_stats.get(&team) {
        Some(stats) => {
            to_value(stats).map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
        }
        None => {
            // Return empty PathStatistics for teams with no knockout appearances
            use wc_simulation::PathStatistics;
            let empty_stats = PathStatistics::new(team);
            to_value(&empty_stats).map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
        }
    }
}
