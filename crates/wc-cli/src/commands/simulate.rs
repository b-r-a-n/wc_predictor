//! Simulate command implementation.

use wc_core::Tournament;
use wc_simulation::{SimulationConfig, SimulationRunner};
use wc_strategies::{CompositeStrategy, EloStrategy, FifaRankingStrategy, FormStrategy, MarketValueStrategy, PredictionStrategy};

use crate::cli::{OutputFormat, SimulateArgs, StrategyChoice};
use crate::error::Result;
use crate::output::{render_simulation_table, Output, SimulationJsonOutput};

pub fn run_simulate(args: &SimulateArgs, tournament: &Tournament, format: OutputFormat) -> Result<()> {
    let output = Output::new(format);

    // Build simulation config
    let mut config = SimulationConfig::with_iterations(args.iterations);
    if let Some(seed) = args.seed {
        config = config.with_seed(seed);
    }
    if let Some(threads) = args.threads {
        config = config.with_parallelism(threads);
    }

    // Create strategy based on choice
    let strategy: Box<dyn PredictionStrategy> = match args.strategy {
        StrategyChoice::Elo => Box::new(EloStrategy::default()),
        StrategyChoice::Fifa => Box::new(FifaRankingStrategy::default()),
        StrategyChoice::Market => Box::new(MarketValueStrategy::default()),
        StrategyChoice::Form => Box::new(FormStrategy::default()),
        StrategyChoice::Composite => {
            Box::new(
                CompositeStrategy::new("Composite")
                    .add_strategy(EloStrategy::default(), args.elo_weight)
                    .add_strategy(MarketValueStrategy::default(), args.market_weight)
                    .add_strategy(FifaRankingStrategy::default(), args.fifa_weight)
                    .add_strategy(FormStrategy::default(), args.form_weight),
            )
        }
    };

    // Run simulation
    let runner = SimulationRunner::new(tournament, strategy.as_ref(), config);
    let results = runner.run();

    // Output results
    if output.is_json() {
        if args.raw {
            // Emit the full AggregatedResults struct as JSON — matches the shape
            // that wasm-bindgen produces so the web app can load it directly.
            output.print_json(&results);
        } else {
            let json_output = SimulationJsonOutput::from_results(&results, tournament);
            output.print_json(&json_output);
        }
    } else {
        render_simulation_table(&results, args.top, tournament);
    }

    Ok(())
}
