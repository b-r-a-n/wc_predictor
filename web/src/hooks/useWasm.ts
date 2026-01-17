import { useEffect, useState } from 'react';
import init, { WcSimulator, calculateMatchProbability, getVersion } from '../../wasm-pkg';
import type { TournamentData, Team, Group, AggregatedResults, MatchProbabilities, CompositeWeights, Strategy } from '../types';

export interface WasmApi {
  simulator: WcSimulator | null;
  teams: Team[];
  groups: Group[];
  version: string;
  runSimulation: (
    strategy: Strategy,
    iterations: number,
    compositeWeights?: CompositeWeights
  ) => AggregatedResults;
  calculateMatchProbability: (
    teamAElo: number,
    teamBElo: number,
    isKnockout: boolean
  ) => MatchProbabilities;
}

export type WasmStatus = 'loading' | 'ready' | 'error';

export function useWasm(): { status: WasmStatus; api: WasmApi | null; error: string | null } {
  const [status, setStatus] = useState<WasmStatus>('loading');
  const [api, setApi] = useState<WasmApi | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function initializeWasm() {
      try {
        // Initialize WASM module
        await init();
        console.log('WASM initialized');

        // Load tournament data
        const response = await fetch(`${import.meta.env.BASE_URL}data/teams.json`);
        if (!response.ok) {
          throw new Error(`Failed to load teams.json: ${response.status}`);
        }
        const tournamentData: TournamentData = await response.json();

        // Create simulator instance
        const simulator = new WcSimulator(tournamentData);
        const teams = simulator.getTeams() as Team[];
        const groups = simulator.getGroups() as Group[];
        const version = getVersion();

        console.log(`Loaded ${teams.length} teams in ${groups.length} groups (v${version})`);

        if (cancelled) return;

        const wasmApi: WasmApi = {
          simulator,
          teams,
          groups,
          version,
          runSimulation: (strategy, iterations, compositeWeights) => {
            let rawResult: unknown;
            switch (strategy) {
              case 'elo':
                rawResult = simulator.runEloSimulation(iterations);
                break;
              case 'market_value':
                rawResult = simulator.runMarketValueSimulation(iterations);
                break;
              case 'fifa_ranking':
                rawResult = simulator.runFifaRankingSimulation(iterations);
                break;
              case 'form':
                rawResult = simulator.runFormSimulation(iterations);
                break;
              case 'composite': {
                const weights = compositeWeights ?? { elo: 0.35, market: 0.25, fifa: 0.25, form: 0.15 };
                rawResult = simulator.runCompositeSimulation(
                  weights.elo,
                  weights.market,
                  weights.fifa,
                  weights.form,
                  iterations
                );
                break;
              }
              default:
                throw new Error(`Unknown strategy: ${strategy}`);
            }

            // Convert Map to plain object (serde-wasm-bindgen returns HashMap as JS Map)
            const result = rawResult as {
              total_simulations: number;
              team_stats: Map<number, unknown> | Record<string, unknown>;
              most_likely_winner: number;
              most_likely_final: [number, number];
            };

            // If team_stats is a Map, convert it to a plain object
            let teamStats: Record<string, unknown>;
            if (result.team_stats instanceof Map) {
              teamStats = {};
              result.team_stats.forEach((value, key) => {
                teamStats[String(key)] = value;
              });
            } else {
              teamStats = result.team_stats as Record<string, unknown>;
            }

            // Get path_stats if available (may be a Map that needs conversion)
            let pathStats: Record<string, unknown> | undefined;
            const rawPathStats = (rawResult as any).path_stats;
            if (rawPathStats instanceof Map) {
              pathStats = {};
              rawPathStats.forEach((value: unknown, key: number) => {
                pathStats![String(key)] = value;
              });
            } else if (rawPathStats) {
              pathStats = rawPathStats as Record<string, unknown>;
            }

            // Get bracket_slot_stats if available (may be a Map that needs conversion)
            let bracketSlotStats: Record<string, unknown> | undefined;
            const rawBracketSlotStats = (rawResult as any).bracket_slot_stats;
            if (rawBracketSlotStats instanceof Map) {
              bracketSlotStats = {};
              rawBracketSlotStats.forEach((value: unknown, key: number) => {
                bracketSlotStats![String(key)] = value;
              });
            } else if (rawBracketSlotStats) {
              bracketSlotStats = rawBracketSlotStats as Record<string, unknown>;
            }

            return {
              total_simulations: result.total_simulations,
              team_stats: teamStats,
              most_likely_winner: result.most_likely_winner,
              most_likely_final: result.most_likely_final,
              path_stats: pathStats,
              bracket_slot_stats: bracketSlotStats,
            } as AggregatedResults;
          },
          calculateMatchProbability: (teamAElo, teamBElo, isKnockout) => {
            return calculateMatchProbability(teamAElo, teamBElo, isKnockout) as MatchProbabilities;
          },
        };

        setApi(wasmApi);
        setStatus('ready');
      } catch (err) {
        console.error('Failed to initialize WASM:', err);
        if (!cancelled) {
          setError(err instanceof Error ? err.message : 'Unknown error');
          setStatus('error');
        }
      }
    }

    initializeWasm();

    return () => {
      cancelled = true;
    };
  }, []);

  return { status, api, error };
}
