import { create } from 'zustand';
import type { Team, Group, AggregatedResults, Strategy, CompositeWeights, TabId, WasmStatus } from '../types';
import type { WasmApi } from '../hooks/useWasm';

interface SimulatorState {
  // WASM state
  wasmStatus: WasmStatus;
  wasmError: string | null;
  wasmApi: WasmApi | null;
  teams: Team[];
  originalTeams: Team[];
  groups: Group[];

  // Team editing state
  editedTeamIds: Set<number>;
  teamsModified: boolean;

  // Simulation settings
  strategy: Strategy;
  iterations: number;
  compositeWeights: CompositeWeights;

  // Simulation state
  isSimulating: boolean;
  results: AggregatedResults | null;

  // UI state
  activeTab: TabId;

  // Actions
  setWasmStatus: (status: WasmStatus, error?: string | null) => void;
  setWasmApi: (api: WasmApi) => void;
  setStrategy: (strategy: Strategy) => void;
  setIterations: (iterations: number) => void;
  setCompositeWeights: (weights: CompositeWeights) => void;
  setActiveTab: (tab: TabId) => void;
  runSimulation: () => void;

  // Team editing actions
  updateTeam: (teamId: number, field: keyof Team, value: number) => void;
  resetTeam: (teamId: number) => void;
  resetAllTeams: () => void;
  reinitializeSimulator: () => Promise<void>;
}

export const useSimulatorStore = create<SimulatorState>((set, get) => ({
  // Initial WASM state
  wasmStatus: 'loading',
  wasmError: null,
  wasmApi: null,
  teams: [],
  originalTeams: [],
  groups: [],

  // Initial team editing state
  editedTeamIds: new Set<number>(),
  teamsModified: false,

  // Initial simulation settings
  strategy: 'elo',
  iterations: 10000,
  compositeWeights: { elo: 0.4, market: 0.3, fifa: 0.3 },

  // Initial simulation state
  isSimulating: false,
  results: null,

  // Initial UI state
  activeTab: 'results',

  // Actions
  setWasmStatus: (status, error = null) => set({ wasmStatus: status, wasmError: error }),

  setWasmApi: (api) => {
    const { originalTeams } = get();
    // Only set originalTeams if it's empty (first initialization)
    const newOriginalTeams = originalTeams.length === 0
      ? api.teams.map(t => ({ ...t }))
      : originalTeams;

    set({
      wasmApi: api,
      teams: api.teams,
      originalTeams: newOriginalTeams,
      groups: api.groups,
      wasmStatus: 'ready',
    });
  },

  setStrategy: (strategy) => set({ strategy }),

  setIterations: (iterations) => set({ iterations }),

  setCompositeWeights: (weights) => set({ compositeWeights: weights }),

  setActiveTab: (tab) => set({ activeTab: tab }),

  runSimulation: () => {
    const { wasmApi, teamsModified, strategy, iterations, compositeWeights, isSimulating, reinitializeSimulator } = get();

    if (!wasmApi || isSimulating) return;

    set({ isSimulating: true });

    // If teams have been modified, reinitialize the simulator first
    const runSim = async () => {
      try {
        if (teamsModified) {
          await reinitializeSimulator();
        }

        const { wasmApi: currentApi } = get();
        if (!currentApi) {
          throw new Error('WASM API not available');
        }

        const results = currentApi.runSimulation(strategy, iterations, compositeWeights);
        set({ results, isSimulating: false });
      } catch (error) {
        console.error('Simulation error:', error);
        set({ isSimulating: false });
      }
    };

    // Use setTimeout to allow UI to update before blocking simulation
    setTimeout(() => {
      runSim();
    }, 50);
  },

  // Team editing actions
  updateTeam: (teamId, field, value) => {
    const { teams, originalTeams, editedTeamIds } = get();

    const updatedTeams = teams.map((team) =>
      team.id === teamId ? { ...team, [field]: value } : team
    );

    // Check if the team is now different from the original
    const originalTeam = originalTeams.find((t) => t.id === teamId);
    const updatedTeam = updatedTeams.find((t) => t.id === teamId);
    const newEditedIds = new Set(editedTeamIds);

    if (originalTeam && updatedTeam) {
      const isDifferent =
        originalTeam.elo_rating !== updatedTeam.elo_rating ||
        originalTeam.market_value_millions !== updatedTeam.market_value_millions ||
        originalTeam.fifa_ranking !== updatedTeam.fifa_ranking;

      if (isDifferent) {
        newEditedIds.add(teamId);
      } else {
        newEditedIds.delete(teamId);
      }
    }

    set({
      teams: updatedTeams,
      editedTeamIds: newEditedIds,
      teamsModified: newEditedIds.size > 0,
    });
  },

  resetTeam: (teamId) => {
    const { teams, originalTeams, editedTeamIds } = get();
    const originalTeam = originalTeams.find((t) => t.id === teamId);

    if (!originalTeam) return;

    const updatedTeams = teams.map((team) =>
      team.id === teamId ? { ...originalTeam } : team
    );

    const newEditedIds = new Set(editedTeamIds);
    newEditedIds.delete(teamId);

    set({
      teams: updatedTeams,
      editedTeamIds: newEditedIds,
      teamsModified: newEditedIds.size > 0,
    });
  },

  resetAllTeams: () => {
    const { originalTeams } = get();

    set({
      teams: originalTeams.map((t) => ({ ...t })),
      editedTeamIds: new Set<number>(),
      teamsModified: false,
    });
  },

  reinitializeSimulator: async () => {
    const { teams, groups } = get();

    try {
      // Dynamic import to avoid circular dependencies
      const { WcSimulator } = await import('../../wasm-pkg');

      // Create tournament data from current teams and groups
      const tournamentData = { teams, groups };

      // Create new simulator instance
      const simulator = new WcSimulator(tournamentData);
      const newTeams = simulator.getTeams() as Team[];
      const newGroups = simulator.getGroups() as Group[];

      // Get version from the existing API or fetch it
      const { wasmApi } = get();
      const version = wasmApi?.version || 'unknown';

      const newApi = {
        simulator,
        teams: newTeams,
        groups: newGroups,
        version,
        runSimulation: (strategy: Strategy, iterations: number, compositeWeights?: CompositeWeights) => {
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
            case 'composite': {
              const weights = compositeWeights ?? { elo: 0.4, market: 0.3, fifa: 0.3 };
              rawResult = simulator.runCompositeSimulation(
                weights.elo,
                weights.market,
                weights.fifa,
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

          return {
            total_simulations: result.total_simulations,
            team_stats: teamStats,
            most_likely_winner: result.most_likely_winner,
            most_likely_final: result.most_likely_final,
          } as AggregatedResults;
        },
        calculateMatchProbability: wasmApi?.calculateMatchProbability || (() => ({ home_win: 0, draw: 0, away_win: 0 })),
      };

      set({
        wasmApi: newApi,
        teamsModified: false,
      });

      console.log('Simulator re-initialized with updated team data');
    } catch (error) {
      console.error('Failed to reinitialize simulator:', error);
      throw error;
    }
  },
}));

// Selector hooks for common data
export const useTeams = () => useSimulatorStore((state) => state.teams);
export const useGroups = () => useSimulatorStore((state) => state.groups);
export const useResults = () => useSimulatorStore((state) => state.results);
export const useIsSimulating = () => useSimulatorStore((state) => state.isSimulating);
