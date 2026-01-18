# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

World Cup 2026 Monte Carlo Simulator - a high-performance prediction system built with Rust/WebAssembly and a React frontend. Runs up to 100,000 tournament simulations in seconds.

**Live demo**: https://b-r-a-n.github.io/wc_predictor/

## Build Commands

### WASM Module (must build first)
```bash
cd crates/wc-wasm
wasm-pack build --target web --release --out-dir ../../web/wasm-pkg
```

### Web Frontend
```bash
cd web
npm install
npm run dev      # Development server at localhost:5173/wc_predictor/
npm run build    # Production build to web/dist/
npm run lint     # ESLint
```

### Rust
```bash
cargo build --workspace
cargo test --workspace
cargo test -p wc-core           # Single crate
cargo test team::tests          # Single test module
```

### WASM Tests
```bash
cd crates/wc-wasm
wasm-pack test --headless --chrome
```

### Python Scrapers
```bash
cd scrapers
python -m venv .venv && source .venv/bin/activate
pip install -r requirements.txt

# From repo root with PYTHONPATH=$PWD:
python -m scrapers.cli.scrape_elo -v
python -m scrapers.cli.scrape_transfermarkt
python -m scrapers.cli.scrape_fifa
python -m scrapers.cli.scrape_sofascore
python -m scrapers.cli.merge_data \
  -m scrapers/config/team_mapping.json \
  -e scrapers/output/elo_ratings.json \
  -t scrapers/output/transfermarkt_values.json \
  -f scrapers/output/fifa_rankings.json \
  -s scrapers/output/sofascore_form.json \
  -g scrapers/output/groups.json \
  -o data/teams.json \
  --allow-tbd-defaults --allow-missing-fifa
python -m scrapers.cli.validate data/teams.json --summary
cp data/teams.json web/public/data/teams.json
```

## Architecture

### Crate Dependency Hierarchy
```
wc-wasm (WASM bindings)
    └── wc-simulation (Monte Carlo engine)
            ├── wc-strategies (prediction algorithms)
            │       └── wc-core (domain types)
            └── wc-core
```

### Key Components

**wc-core**: Pure domain types - `Team`, `Group`, `MatchResult`, `Tournament`, `KnockoutBracket`. No business logic, just serde-serializable data structures.

**wc-strategies**: Trait-based prediction system via `PredictionStrategy`:
- `EloStrategy` - Standard ELO formula with home advantage
- `MarketValueStrategy` - Log-scale squad value comparison
- `FifaRankingStrategy` - Sqrt-compressed ranking scores
- `FormStrategy` - Sofascore recent form ratings
- `CompositeStrategy` - Weighted blend (default: ELO 35%, Market 25%, FIFA 25%, Form 15%)

**wc-simulation**:
- `SimulationEngine` - Runs single tournament with group stage + knockout
- `SimulationRunner` - Parallel batch runner using rayon (sequential in WASM)
- `Aggregator` - Collects statistics across all simulations

**wc-wasm**: JavaScript FFI layer exposing `WcSimulator` class with `runEloSimulation()`, `runCompositeSimulation()`, etc.

**web/**: React 19 + Vite 7 + TypeScript + Tailwind CSS 4
- State: Zustand store in `src/store/simulatorStore.ts`
- WASM init: `src/hooks/useWasm.ts`
- Team edits persist to LocalStorage; reinitializes WASM simulator when modified

### Data Flow
1. `teams.json` loaded by WASM → `WcSimulator` instance
2. User selects strategy/iterations in UI
3. WASM runs parallel simulations → returns `AggregatedResults`
4. Results displayed across tabs (Win Probability, Groups, Bracket, Paths, Calculator)

## Important Patterns

**Seeded RNG**: Uses `ChaCha8Rng` for reproducibility. Base seed + iteration offset enables deterministic parallel runs.

**WASM Map Conversion**: `serde-wasm-bindgen` returns JavaScript `Map` objects. The frontend converts these to plain objects for React state.

**Lazy Reinitializer**: When teams are edited in the UI, a new `WcSimulator` instance is created only before the next simulation run, not on every edit.

## Tournament Format (2026)

- 12 groups × 4 teams (round-robin)
- Top 2 per group (24) + 8 best 3rd place = 32 advance
- Knockout: R32 → R16 → QF → SF → Final
- Tiebreakers: Points → GD → Goals → H2H → Fair play → Lots

## Deployment

GitHub Actions deploys to GitHub Pages on push to main. Base path is `/wc_predictor/`.
