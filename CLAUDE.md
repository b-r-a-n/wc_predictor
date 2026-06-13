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

# Completed match results (already-played group-stage games) from ESPN.
# Re-run daily as more matches finish; output is a full snapshot every time.
# Defaults to the committed web/public/data/schedule.json and the group draw
# in scrapers/config/team_mapping.json, so no other scraper output is required.
# (The `.github/workflows/scrape-results.yml` cron runs this daily in CI and
# commits the result — see Deployment.)
python -m scrapers.cli.scrape_results -v
cp scrapers/output/results.json web/public/data/results.json

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

**Real Match Results**: `scrape_results` produces `web/public/data/results.json` (completed group-stage scores mapped to schedule `matchNumber`s). The store loads these into `actualResults` and, when the "Start from real results" toggle is on (`useActualResults`, default on), merges them as the base layer beneath the user's manual locks (`fixedResults`) — see `effectiveFixedResults` in `simulatorStore.ts`. Both flow through the existing `setFixedResults` fixture-locking path, so the simulator needs no changes. Only group-stage results are supported (knockout fixtures use positional placeholders the fixed-result path doesn't handle).

## Tournament Format (2026)

- 12 groups × 4 teams (round-robin)
- Top 2 per group (24) + 8 best 3rd place = 32 advance
- Knockout: R32 → R16 → QF → SF → Final
- Tiebreakers: Points → GD → Goals → H2H → Fair play → Lots

## Deployment

GitHub Actions deploys to GitHub Pages on push to main. Base path is `/wc_predictor/`.

`scrape-results.yml` runs daily (09:00 UTC) and on demand (`workflow_dispatch`), scraping completed group-stage results into `web/public/data/results.json`, committing changes, and dispatching `deploy.yml`. Because pushes made with `GITHUB_TOKEN` don't trigger other workflows, the deploy is kicked off explicitly via `gh workflow run`. No always-on machine is needed.
