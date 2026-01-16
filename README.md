# World Cup 2026 Monte Carlo Simulator

A high-performance Monte Carlo simulator for predicting FIFA World Cup 2026 outcomes. Built with Rust and WebAssembly for near-native speed in the browser, featuring multiple prediction strategies and an interactive React UI.

![Screenshot Placeholder](docs/screenshot.png)

## Live Demo

**[Try it now: https://b-r-a-n.github.io/wc_predictor/](https://b-r-a-n.github.io/wc_predictor/)**

## Features

- **Multiple Prediction Strategies** - ELO ratings, market values, FIFA rankings, or a weighted composite
- **High-Performance Simulation** - Run up to 100,000 Monte Carlo iterations in seconds using WebAssembly
- **Win Probability Table** - Sortable rankings showing tournament win probability for all 48 teams
- **Group Stage View** - 12 groups with advancement probabilities for each position
- **Knockout Bracket** - Visual bracket showing progression probabilities through each round
- **Head-to-Head Calculator** - Compare any two teams with detailed match outcome probabilities
- **Team Data Editor** - Customize ELO ratings, market values, and FIFA rankings with LocalStorage persistence
- **Named Presets** - Save and load custom team configurations
- **Reproducible Results** - Seed-based RNG for consistent simulation outputs

## Tournament Format (2026)

The 2026 FIFA World Cup expands to 48 teams with a new format:

| Stage | Details |
|-------|---------|
| **Group Stage** | 12 groups of 4 teams, round-robin (3 matches each) |
| **Advancement** | Top 2 from each group (24) + 8 best third-place teams = 32 |
| **Knockout** | Round of 32 -> Round of 16 -> Quarterfinals -> Semifinals -> Final |

The simulator implements FIFA-compliant tiebreaker rules:
1. Points
2. Goal difference
3. Goals scored
4. Head-to-head results
5. Fair play points
6. Drawing of lots

## Prediction Algorithms

### ELO Rating Strategy

Based on the World Football ELO rating system. Calculates win expectancy using the standard formula:

```
We = 1 / (10^(-dr/400) + 1)
```

Where `dr` is the rating difference between teams. Features:
- Home advantage adjustment (~100 ELO points)
- Draw probability peaks at ~28% for evenly matched teams
- Poisson-distributed goal scoring with base rate of 1.3 goals/team

### Market Value Strategy

Uses squad market values with log-scale comparison to prevent extreme probability ratios:

```
ratio = ln(home_value + 1) / (ln(home_value + 1) + ln(away_value + 1))
```

This ensures underdogs always retain meaningful upset potential even against much wealthier squads.

### FIFA Ranking Strategy

Converts FIFA world rankings to strength scores using square root compression:

```
strength = 1 - sqrt(ranking / max_ranking)
```

Lower rankings (e.g., #1) produce higher strength scores. Rankings beyond 100 are capped to normalize probabilities.

### Composite Strategy

Weighted combination of all three strategies with configurable weights:

```
composite = (elo_weight * elo_prob) + (market_weight * market_prob) + (fifa_weight * fifa_prob)
```

Default weights: ELO 50%, Market Value 30%, FIFA Ranking 20%

## Tech Stack

### Backend (Rust)
- **serde** - JSON serialization/deserialization
- **rand / rand_chacha** - Reproducible random number generation
- **rayon** - Parallel iteration for native builds
- **wasm-bindgen** - WebAssembly bindings
- **serde-wasm-bindgen** - Efficient Rust <-> JS data transfer

### Frontend (React/TypeScript)
- **React 19** - UI framework
- **Vite 7** - Build tooling with WASM support
- **TypeScript 5.9** - Type safety
- **Tailwind CSS 4** - Styling
- **Zustand** - State management

### Data Scrapers (Python)
- **requests / cloudscraper** - HTTP with Cloudflare bypass
- **beautifulsoup4 / lxml** - HTML parsing
- **pydantic** - Data validation
- **click / rich** - CLI interface

## Getting Started

### Prerequisites

- **Rust** 1.70+ with `wasm32-unknown-unknown` target
- **Node.js** 18+ and npm
- **wasm-pack** for building WASM
- **Python** 3.11+ (for data scrapers only)

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/b-r-a-n/wc_predictor.git
   cd wc_predictor
   ```

2. **Install wasm-pack** (if not already installed)
   ```bash
   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
   ```

3. **Build the WASM module**
   ```bash
   cd crates/wc-wasm
   wasm-pack build --target web --out-dir ../../web/wasm-pkg
   cd ../..
   ```

4. **Install web dependencies**
   ```bash
   cd web
   npm install
   ```

5. **Start the development server**
   ```bash
   npm run dev
   ```

6. **Open the app**
   Navigate to [http://localhost:5173](http://localhost:5173)

### Running Tests

```bash
# Rust unit tests
cargo test --workspace

# WASM tests (requires browser or Node.js)
cd crates/wc-wasm
wasm-pack test --headless --chrome
```

## Data Scrapers

The `scrapers/` directory contains Python scripts to fetch fresh team data from various sources.

### Setup

```bash
cd scrapers
python -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate
pip install -r requirements.txt
```

### Data Sources

| Source | URL | Data |
|--------|-----|------|
| ELO Ratings | international-football.net | Team ELO scores |
| Market Values | transfermarkt.us | Squad values (millions EUR) |
| FIFA Rankings | wikipedia.org | Official FIFA rankings |
| Groups | team_mapping.json | Official December 2025 draw |

### Fetching Fresh Data

```bash
cd /path/to/wc_predictor
source scrapers/.venv/bin/activate
export PYTHONPATH=$PWD

# Run individual scrapers (each takes 1-2 minutes due to rate limiting)
python -m scrapers.cli.scrape_elo -v
python -m scrapers.cli.scrape_transfermarkt
python -m scrapers.cli.scrape_fifa
python -m scrapers.cli.scrape_groups

# Merge all data sources into final teams.json
python -m scrapers.cli.merge_data \
  -m scrapers/config/team_mapping.json \
  -e scrapers/output/elo_ratings.json \
  -t scrapers/output/transfermarkt_values.json \
  -f scrapers/output/fifa_rankings.json \
  -g scrapers/output/groups.json \
  -o data/teams.json \
  --allow-tbd-defaults \
  --allow-missing-fifa

# Validate the output (runs 14 automated checks)
python -m scrapers.cli.validate data/teams.json --summary

# Copy to web assets
cp data/teams.json web/public/data/teams.json
```

## Project Structure

```
wc_predictor/
├── Cargo.toml                    # Rust workspace configuration
├── data/
│   └── teams.json                # 48 teams with all ratings and group assignments
├── crates/
│   ├── wc-core/                  # Core domain types
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── team.rs           # Team, TeamId, Confederation
│   │       ├── match_result.rs   # MatchResult, goals, penalties
│   │       ├── group.rs          # Group, GroupResult, standings
│   │       ├── knockout.rs       # Bracket, KnockoutMatch
│   │       ├── tournament.rs     # Tournament configuration
│   │       └── tiebreaker.rs     # FIFA tiebreaker rules
│   ├── wc-strategies/            # Prediction algorithms
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── traits.rs         # PredictionStrategy trait
│   │       ├── elo.rs            # ELO-based predictions
│   │       ├── market_value.rs   # Market value predictions
│   │       ├── fifa_ranking.rs   # FIFA ranking predictions
│   │       └── composite.rs      # Weighted combination
│   ├── wc-simulation/            # Monte Carlo engine
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── engine.rs         # Single tournament simulation
│   │       ├── runner.rs         # Parallel Monte Carlo runner
│   │       └── aggregator.rs     # Results aggregation
│   └── wc-wasm/                  # WebAssembly bindings
│       └── src/
│           ├── lib.rs
│           └── api.rs            # JavaScript API
├── scrapers/                     # Python data scrapers
│   ├── requirements.txt
│   ├── config/
│   │   ├── settings.py           # Paths, timeouts, rate limits
│   │   └── team_mapping.json     # 48 WC 2026 teams with aliases
│   ├── sources/
│   │   ├── base.py               # BaseScraper class
│   │   ├── elo_scraper.py
│   │   ├── transfermarkt_scraper.py
│   │   ├── fifa_scraper.py
│   │   └── groups_scraper.py
│   ├── models/
│   │   └── team.py               # Pydantic validation
│   ├── cli/
│   │   ├── scrape_elo.py
│   │   ├── scrape_transfermarkt.py
│   │   ├── scrape_fifa.py
│   │   ├── scrape_groups.py
│   │   ├── merge_data.py
│   │   └── validate.py
│   └── output/                   # Intermediate data (gitignored)
└── web/                          # React frontend
    ├── package.json
    ├── vite.config.ts
    ├── wasm-pkg/                 # Built WASM module
    ├── public/
    │   └── data/teams.json       # Team data for web app
    └── src/
        ├── main.tsx
        ├── App.tsx
        ├── index.css
        ├── components/
        │   ├── common/           # Button, Card, LoadingSpinner, ProbabilityBar
        │   ├── controls/         # ControlPanel, StrategySelector, CompositeWeights
        │   ├── results/          # WinProbabilityTable
        │   ├── groups/           # GroupStageView, GroupTable
        │   ├── bracket/          # KnockoutBracket, BracketMatch, TeamSlot
        │   ├── calculator/       # HeadToHeadCalculator, TeamSelector
        │   ├── editor/           # TeamDataEditor
        │   └── layout/           # Header, Layout, TabNavigation
        ├── hooks/
        │   └── useWasm.ts        # WASM initialization hook
        ├── store/
        │   └── simulatorStore.ts # Zustand state management
        ├── types/
        │   ├── index.ts
        │   └── wasm.d.ts
        └── utils/
            └── formatting.ts
```

## Building for Production

### Build WASM with Optimizations

```bash
cd crates/wc-wasm
wasm-pack build --target web --release --out-dir ../../web/wasm-pkg
```

### Build Web App

```bash
cd web
npm run build
```

The production build will be in `web/dist/`.

### Deploy to GitHub Pages

```bash
# From the web directory after building
npm run build
# Push dist/ contents to gh-pages branch
```

## WASM API Reference

The WebAssembly module exposes a `WcSimulator` class:

```typescript
class WcSimulator {
  // Create a new simulator with tournament configuration
  constructor(tournamentJson: TournamentConfig);

  // Run simulation with ELO strategy
  runEloSimulation(iterations: number, seed?: bigint): AggregatedResults;

  // Run simulation with market value strategy
  runMarketValueSimulation(iterations: number, seed?: bigint): AggregatedResults;

  // Run simulation with FIFA ranking strategy
  runFifaRankingSimulation(iterations: number, seed?: bigint): AggregatedResults;

  // Run simulation with composite strategy (custom weights)
  runCompositeSimulation(
    eloWeight: number,
    marketWeight: number,
    fifaWeight: number,
    iterations: number,
    seed?: bigint
  ): AggregatedResults;

  // Get team list
  getTeams(): Team[];

  // Get group assignments
  getGroups(): Group[];
}
```

## Known Limitations

- **FIFA Rankings**: Only top 20 rankings available from Wikipedia; lower-ranked teams use estimates derived from ELO ratings
- **TBD Playoff Teams**: 6 spots use placeholder values until the March 2026 intercontinental playoffs determine qualifiers
- **Transfermarkt Scraping**: May require cloudscraper updates if Cloudflare protection changes

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

Built with Rust and WebAssembly for the FIFA World Cup 2026.
