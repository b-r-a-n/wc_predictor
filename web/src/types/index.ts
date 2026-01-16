// Team and tournament types matching Rust structs

export type Confederation = 'UEFA' | 'CONMEBOL' | 'CONCACAF' | 'CAF' | 'AFC' | 'OFC';

export interface Team {
  id: number;
  name: string;
  code: string;
  confederation: Confederation;
  elo_rating: number;
  market_value_millions: number;
  fifa_ranking: number;
  world_cup_wins: number;
}

export interface Group {
  id: string;
  teams: number[];
}

export interface TournamentData {
  teams: Team[];
  groups: Group[];
}

// Simulation result types matching Rust aggregator

export interface TeamStatistics {
  team_id: number;
  team_name: string;

  // Group stage outcomes
  group_wins: number;
  group_second: number;
  group_third_qualified: number;
  group_eliminated: number;

  // Knockout rounds reached
  reached_round_of_32: number;
  reached_round_of_16: number;
  reached_quarter_finals: number;
  reached_semi_finals: number;
  reached_final: number;

  // Final positions
  champion: number;
  runner_up: number;
  third_place: number;
  fourth_place: number;

  // Calculated probabilities
  win_probability: number;
  final_probability: number;
  semi_final_probability: number;
  knockout_probability: number;
}

export interface AggregatedResults {
  total_simulations: number;
  team_stats: Record<string, TeamStatistics>;  // Keys are stringified numbers due to JSON serialization
  most_likely_winner: number;
  most_likely_final: [number, number];
}

// Match probability types
export interface MatchProbabilities {
  home_win: number;
  draw: number;
  away_win: number;
}

// Strategy types
export type Strategy = 'elo' | 'market_value' | 'fifa_ranking' | 'composite';

export interface CompositeWeights {
  elo: number;
  market: number;
  fifa: number;
}

// UI state types
export type WasmStatus = 'loading' | 'ready' | 'error';
export type TabId = 'results' | 'groups' | 'bracket' | 'calculator' | 'editor';

// Team preset for LocalStorage persistence
export interface TeamPreset {
  name: string;
  teams: Team[];
  createdAt: number;  // timestamp
}
