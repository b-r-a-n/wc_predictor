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
  sofascore_form: number;
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
  path_stats?: Record<string, PathStatistics>;  // Path statistics for each team (keys are stringified TeamIds)
  bracket_slot_stats?: Record<string, BracketSlotStats>;  // Bracket slot statistics for each team (keys are stringified TeamIds)
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
export type TabId = 'results' | 'groups' | 'bracket' | 'calculator' | 'editor' | 'paths';

// Knockout round types
export type KnockoutRoundType =
  | 'round_of_32'
  | 'round_of_16'
  | 'quarter_finals'
  | 'semi_finals'
  | 'third_place'
  | 'final';

// Venue information for World Cup 2026
export interface Venue {
  id: string;
  name: string;
  city: string;
  country: 'USA' | 'Canada' | 'Mexico';
  hostingRounds: KnockoutRoundType[];
}

// Venue data file structure
export interface VenueData {
  venues: Venue[];
  bracketVenueMapping: Record<KnockoutRoundType, Record<string, string>>;
}

// Matchup frequencies at a specific round (matches Rust RoundMatchups)
export interface RoundMatchups {
  opponents: Record<string, number>; // opponent_id -> count
}

// Path statistics for a team (matches Rust PathStatistics)
export interface PathStatistics {
  team_id: number;
  round_of_32_matchups: RoundMatchups;
  round_of_16_matchups: RoundMatchups;
  quarter_final_matchups: RoundMatchups;
  semi_final_matchups: RoundMatchups;
  final_matchups: RoundMatchups;
  complete_paths: Record<string, number>; // path_key -> count
}

// Bracket slot statistics - tracks which bracket positions each team occupies
export interface BracketSlotStats {
  round_of_32: Record<string, number>;   // slot "0"-"15" -> count
  round_of_16: Record<string, number>;   // slot "0"-"7" -> count
  quarter_finals: Record<string, number>; // slot "0"-"3" -> count
  semi_finals: Record<string, number>;    // slot "0"-"1" -> count
  final_match: number;
}

// Result from getTopPathsForTeam WASM function
export interface PathEntry {
  path: string;
  count: number;
  probability: number;
}

export interface TopPathsResult {
  team_id: number;
  total_simulations: number;
  has_paths: boolean;
  paths: PathEntry[];
}

// Parsed path for display purposes
export interface ParsedPathRound {
  round: KnockoutRoundType;
  roundDisplayName: string;
  opponentId: number;
  opponent?: Team;
  venue?: Venue;
}

// A fully resolved tournament path for display
export interface TournamentPathDisplay {
  rank: number;
  pathKey: string;
  probability: number;
  occurrenceCount: number;
  rounds: ParsedPathRound[];
}

// Team preset for LocalStorage persistence
export interface TeamPreset {
  name: string;
  teams: Team[];
  createdAt: number;  // timestamp
}
