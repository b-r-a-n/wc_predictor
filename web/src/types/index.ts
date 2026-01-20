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

// Slot data from Rust bracket algorithm
export interface RustMostLikelyBracketSlot {
  team_id: number;
  count: number;
  probability: number;
}

// Single R32 match in the optimal bracket with both teams and winner
export interface RustOptimalR32Match {
  slot: number;
  team_a: RustMostLikelyBracketSlot;
  team_b: RustMostLikelyBracketSlot;
  winner: number;  // TeamId
}

// Optimal bracket computed via Hungarian algorithm from Rust
// Guarantees exactly 32 unique teams in R32 and valid bracket structure
export interface RustOptimalBracket {
  round_of_32: RustOptimalR32Match[];  // 16 matches with both teams
  round_of_16: Record<string, RustMostLikelyBracketSlot>;
  quarter_finals: Record<string, RustMostLikelyBracketSlot>;
  semi_finals: Record<string, RustMostLikelyBracketSlot>;
  champion: RustMostLikelyBracketSlot | null;
  joint_probability: number;
  log_probability: number;
}

export interface AggregatedResults {
  total_simulations: number;
  team_stats: Record<string, TeamStatistics>;  // Keys are stringified numbers due to JSON serialization
  most_likely_winner: number;
  most_likely_final: [number, number];
  path_stats?: Record<string, PathStatistics>;  // Path statistics for each team (keys are stringified TeamIds)
  bracket_slot_stats?: Record<string, BracketSlotStats>;  // Bracket slot statistics for each team (keys are stringified TeamIds)
  bracket_slot_win_stats?: Record<string, BracketSlotWinStats>;  // Bracket slot WIN statistics (only winners, not participants)
  slot_opponent_stats?: Record<string, SlotOpponentStats>;  // Slot-specific opponent statistics (keys are stringified TeamIds)
  optimal_bracket?: RustOptimalBracket;  // Hungarian algorithm bracket (exactly 32 unique teams)
}

// Match probability types
export interface MatchProbabilities {
  home_win: number;
  draw: number;
  away_win: number;
}

// Strategy types
export type Strategy = 'elo' | 'market_value' | 'fifa_ranking' | 'form' | 'composite';

export interface CompositeWeights {
  elo: number;
  market: number;
  fifa: number;
  form: number;
}

// UI state types
export type WasmStatus = 'loading' | 'ready' | 'error';
export type TabId = 'results' | 'groups' | 'bracket' | 'editor' | 'venues';

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
  // Note: bracketVenueMapping removed - use schedule.json with matchMapping.ts for venue lookup
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

// Bracket slot statistics - tracks which bracket positions each team occupies (participation)
export interface BracketSlotStats {
  round_of_32: Record<string, number>;   // slot "0"-"15" -> count
  round_of_16: Record<string, number>;   // slot "0"-"7" -> count
  quarter_finals: Record<string, number>; // slot "0"-"3" -> count
  semi_finals: Record<string, number>;    // slot "0"-"1" -> count
  final_match: number;
}

// Bracket slot WIN statistics - tracks only WINS per slot (not participation)
export interface BracketSlotWinStats {
  round_of_32: Record<string, number>;   // slot "0"-"15" -> win count
  round_of_16: Record<string, number>;   // slot "0"-"7" -> win count
  quarter_finals: Record<string, number>; // slot "0"-"3" -> win count
  semi_finals: Record<string, number>;    // slot "0"-"1" -> win count
  final_match: number;
}

// Slot-specific opponent statistics - tracks which opponents were faced in each specific bracket slot
export interface SlotOpponentStats {
  round_of_32: Record<string, Record<string, number>>;  // slot -> opponent_id -> count
  round_of_16: Record<string, Record<string, number>>;
  quarter_finals: Record<string, Record<string, number>>;
  semi_finals: Record<string, Record<string, number>>;
  final_match: Record<string, number>;  // opponent_id -> count (single slot)
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

// Schedule types for FIFA World Cup 2026 match schedule
export interface ScheduledMatch {
  matchNumber: number;
  date: string;              // "2026-06-11"
  time: string;              // "13:00"
  venueId: string;
  round: 'group_stage' | KnockoutRoundType;
  groupId?: string;          // For group stage matches
  // Note: knockoutSlot removed - derive from matchNumber using utils/matchMapping.ts
  homeTeamId?: number;       // Actual team ID if known
  awayTeamId?: number;       // Actual team ID if known
  homePlaceholder?: string;  // "A1", "1A", "W1", etc.
  awayPlaceholder?: string;
}

export interface MatchScheduleData {
  matches: ScheduledMatch[];
  lastUpdated: string;
  source?: string;
  tournament?: string;
}

// Team probability for displaying candidates in knockout matches
export interface TeamProbability {
  team: Team;
  probability: number;
}

// Most likely bracket slot data - the most probable team for a specific slot
export interface MostLikelySlotData {
  teamId: number;
  team: Team;
  count: number;
  probability: number;
}

// Most likely bracket across all slots
export interface MostLikelyBracket {
  round_of_32: Record<string, MostLikelySlotData>;
  round_of_16: Record<string, MostLikelySlotData>;
  quarter_finals: Record<string, MostLikelySlotData>;
  semi_finals: Record<string, MostLikelySlotData>;
  final_match: MostLikelySlotData | null;
  champion: MostLikelySlotData | null;
}
