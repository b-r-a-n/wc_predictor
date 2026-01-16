/* tslint:disable */
/* eslint-disable */

/**
 * Main simulator interface for JavaScript.
 */
export class WcSimulator {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Get the group configuration.
     */
    getGroups(): any;
    /**
     * Get the list of teams.
     */
    getTeams(): any;
    /**
     * Create a new simulator from tournament JSON data.
     *
     * # Arguments
     * * `tournament_json` - JSON object containing teams and groups
     *
     * # Example
     * ```javascript
     * const simulator = new WcSimulator({
     *   teams: [...],
     *   groups: [...]
     * });
     * ```
     */
    constructor(tournament_json: any);
    /**
     * Get the number of groups.
     */
    numGroups(): number;
    /**
     * Get the number of teams.
     */
    numTeams(): number;
    /**
     * Run simulation using a composite strategy.
     *
     * # Arguments
     * * `elo_weight` - Weight for ELO strategy (0.0 to 1.0)
     * * `market_weight` - Weight for market value strategy
     * * `fifa_weight` - Weight for FIFA ranking strategy
     * * `iterations` - Number of simulations
     * * `seed` - Optional seed
     */
    runCompositeSimulation(elo_weight: number, market_weight: number, fifa_weight: number, iterations: number, seed?: bigint | null): any;
    /**
     * Run simulation using ELO-based predictions.
     *
     * # Arguments
     * * `iterations` - Number of tournament simulations to run
     * * `seed` - Optional seed for reproducibility
     */
    runEloSimulation(iterations: number, seed?: bigint | null): any;
    /**
     * Run simulation using FIFA ranking-based predictions.
     */
    runFifaRankingSimulation(iterations: number, seed?: bigint | null): any;
    /**
     * Run simulation using market value-based predictions.
     */
    runMarketValueSimulation(iterations: number, seed?: bigint | null): any;
}

/**
 * Calculate head-to-head match probability between two teams.
 *
 * Returns an object with home_win, draw, and away_win probabilities.
 */
export function calculateMatchProbability(team_a_elo: number, team_b_elo: number, is_knockout: boolean): any;

/**
 * Get matchup frequencies for a specific team at each knockout round.
 *
 * Returns opponent frequencies at each round of the knockout stage.
 *
 * # Arguments
 * * `results` - The AggregatedResults from a simulation run
 * * `team_id` - The team ID to get matchup stats for
 *
 * # Returns
 * An object containing matchup frequencies for each knockout round.
 */
export function getTeamMatchupStats(results: any, team_id: number): any;

/**
 * Get top N tournament paths for a specific team.
 *
 * This helper function extracts and formats path statistics from simulation results.
 * Useful for visualizing the most common tournament paths a team takes.
 *
 * # Arguments
 * * `results` - The AggregatedResults from a simulation run
 * * `team_id` - The team ID to get paths for
 * * `top_n` - Maximum number of paths to return
 *
 * # Returns
 * A TopPathsResult containing the top N paths sorted by occurrence count.
 *
 * # Example
 * ```javascript
 * const results = simulator.runEloSimulation(10000, undefined);
 * const topPaths = getTopPathsForTeam(results, 0, 10);
 * console.log(topPaths.paths);
 * ```
 */
export function getTopPathsForTeam(results: any, team_id: number, top_n: number): any;

/**
 * Get version information.
 */
export function getVersion(): string;

/**
 * Initialize panic hook for better error messages in browser console.
 */
export function init(): void;

/**
 * Run a single tournament simulation and return detailed results.
 *
 * Useful for step-by-step visualization of a single tournament.
 */
export function simulateSingleTournament(tournament_json: any, strategy: string, seed: bigint): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_wcsimulator_free: (a: number, b: number) => void;
    readonly calculateMatchProbability: (a: number, b: number, c: number) => any;
    readonly getTeamMatchupStats: (a: any, b: number) => [number, number, number];
    readonly getTopPathsForTeam: (a: any, b: number, c: number) => [number, number, number];
    readonly getVersion: () => [number, number];
    readonly simulateSingleTournament: (a: any, b: number, c: number, d: bigint) => [number, number, number];
    readonly wcsimulator_getGroups: (a: number) => [number, number, number];
    readonly wcsimulator_getTeams: (a: number) => [number, number, number];
    readonly wcsimulator_new: (a: any) => [number, number, number];
    readonly wcsimulator_numGroups: (a: number) => number;
    readonly wcsimulator_numTeams: (a: number) => number;
    readonly wcsimulator_runCompositeSimulation: (a: number, b: number, c: number, d: number, e: number, f: number, g: bigint) => [number, number, number];
    readonly wcsimulator_runEloSimulation: (a: number, b: number, c: number, d: bigint) => [number, number, number];
    readonly wcsimulator_runFifaRankingSimulation: (a: number, b: number, c: number, d: bigint) => [number, number, number];
    readonly wcsimulator_runMarketValueSimulation: (a: number, b: number, c: number, d: bigint) => [number, number, number];
    readonly init: () => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
