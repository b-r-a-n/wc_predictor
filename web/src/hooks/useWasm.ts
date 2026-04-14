import { useEffect, useState } from 'react';
import init, { WcSimulator, calculateMatchProbability, getVersion } from '../../wasm-pkg';
import type { TournamentData, Team, Group, AggregatedResults, MatchProbabilities, CompositeWeights, Strategy } from '../types';
import { normalizeSimulationResult } from '../utils/normalizeSimulationResult';

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
                const weights = compositeWeights ?? { elo: 0.4, market: 0.4, fifa: 0.1, form: 0.1 };
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

            return normalizeSimulationResult(rawResult);
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
