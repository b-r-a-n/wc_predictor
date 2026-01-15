// Type declarations for WASM module

declare module '../../wasm-pkg' {
  export class WcSimulator {
    constructor(tournament_json: unknown);

    runEloSimulation(iterations: number, seed?: bigint): unknown;
    runMarketValueSimulation(iterations: number, seed?: bigint): unknown;
    runFifaRankingSimulation(iterations: number, seed?: bigint): unknown;
    runCompositeSimulation(
      elo_weight: number,
      market_weight: number,
      fifa_weight: number,
      iterations: number,
      seed?: bigint
    ): unknown;

    getTeams(): unknown;
    getGroups(): unknown;
    numTeams(): number;
    numGroups(): number;
  }

  export function simulateSingleTournament(
    tournament_json: unknown,
    strategy: string,
    seed: bigint
  ): unknown;

  export function calculateMatchProbability(
    team_a_elo: number,
    team_b_elo: number,
    is_knockout: boolean
  ): unknown;

  export function getVersion(): string;

  export default function init(): Promise<void>;
}
