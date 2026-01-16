import { create } from 'zustand';
import type { Team, Group, AggregatedResults, Strategy, CompositeWeights, TabId, WasmStatus, TeamPreset } from '../types';
import type { WasmApi } from '../hooks/useWasm';

// LocalStorage keys
const STORAGE_KEY_CURRENT_EDITS = 'wc_predictor_current_edits';
const STORAGE_KEY_PRESETS = 'wc_predictor_presets';

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

  // Presets state
  presets: TeamPreset[];
  activePresetName: string | null;

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

  // Persistence actions
  saveCurrentEditsToStorage: () => void;
  loadCurrentEditsFromStorage: () => void;
  loadPresetsFromStorage: () => void;
  savePreset: (name: string) => void;
  loadPreset: (name: string) => void;
  deletePreset: (name: string) => void;
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

  // Initial presets state
  presets: [],
  activePresetName: null,

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
    const { originalTeams, loadCurrentEditsFromStorage, loadPresetsFromStorage } = get();
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

    // Load persisted data from LocalStorage after WASM init
    loadPresetsFromStorage();
    loadCurrentEditsFromStorage();
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
    const { teams, originalTeams, editedTeamIds, saveCurrentEditsToStorage } = get();

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
      activePresetName: null,  // Clear active preset when manually editing
    });

    // Auto-save to LocalStorage
    setTimeout(() => saveCurrentEditsToStorage(), 0);
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
      activePresetName: null,
    });

    // Clear current edits from LocalStorage
    try {
      localStorage.removeItem(STORAGE_KEY_CURRENT_EDITS);
    } catch (e) {
      console.warn('Failed to clear LocalStorage:', e);
    }
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

  // Persistence actions
  saveCurrentEditsToStorage: () => {
    const { teams, editedTeamIds } = get();
    if (editedTeamIds.size === 0) {
      // No edits, remove from storage
      try {
        localStorage.removeItem(STORAGE_KEY_CURRENT_EDITS);
      } catch (e) {
        console.warn('Failed to clear LocalStorage:', e);
      }
      return;
    }

    // Only save edited teams (as a diff)
    const editedTeams = teams.filter(t => editedTeamIds.has(t.id));
    try {
      localStorage.setItem(STORAGE_KEY_CURRENT_EDITS, JSON.stringify(editedTeams));
    } catch (e) {
      console.warn('Failed to save to LocalStorage:', e);
    }
  },

  loadCurrentEditsFromStorage: () => {
    try {
      const stored = localStorage.getItem(STORAGE_KEY_CURRENT_EDITS);
      if (!stored) return;

      const editedTeams: Team[] = JSON.parse(stored);
      const { teams } = get();

      // Apply saved edits to current teams
      const newEditedIds = new Set<number>();
      const updatedTeams = teams.map(team => {
        const savedEdit = editedTeams.find(t => t.id === team.id);
        if (savedEdit) {
          newEditedIds.add(team.id);
          return {
            ...team,
            elo_rating: savedEdit.elo_rating,
            market_value_millions: savedEdit.market_value_millions,
            fifa_ranking: savedEdit.fifa_ranking,
          };
        }
        return team;
      });

      set({
        teams: updatedTeams,
        editedTeamIds: newEditedIds,
        teamsModified: newEditedIds.size > 0,
      });

      console.log(`Restored ${newEditedIds.size} edited teams from LocalStorage`);
    } catch (e) {
      console.warn('Failed to load from LocalStorage:', e);
    }
  },

  loadPresetsFromStorage: () => {
    try {
      const stored = localStorage.getItem(STORAGE_KEY_PRESETS);
      if (!stored) return;

      const presets: TeamPreset[] = JSON.parse(stored);
      set({ presets });
      console.log(`Loaded ${presets.length} presets from LocalStorage`);
    } catch (e) {
      console.warn('Failed to load presets from LocalStorage:', e);
    }
  },

  savePreset: (name: string) => {
    const { teams, presets } = get();

    // Check if preset with same name exists
    const existingIndex = presets.findIndex(p => p.name === name);
    const newPreset: TeamPreset = {
      name,
      teams: teams.map(t => ({ ...t })),
      createdAt: Date.now(),
    };

    let newPresets: TeamPreset[];
    if (existingIndex >= 0) {
      // Update existing preset
      newPresets = [...presets];
      newPresets[existingIndex] = newPreset;
    } else {
      // Add new preset
      newPresets = [...presets, newPreset];
    }

    set({ presets: newPresets, activePresetName: name });

    try {
      localStorage.setItem(STORAGE_KEY_PRESETS, JSON.stringify(newPresets));
    } catch (e) {
      console.warn('Failed to save presets to LocalStorage:', e);
    }
  },

  loadPreset: (name: string) => {
    const { presets, originalTeams } = get();
    const preset = presets.find(p => p.name === name);
    if (!preset) return;

    // Calculate which teams differ from original
    const newEditedIds = new Set<number>();
    preset.teams.forEach(team => {
      const original = originalTeams.find(t => t.id === team.id);
      if (original) {
        const isDifferent =
          original.elo_rating !== team.elo_rating ||
          original.market_value_millions !== team.market_value_millions ||
          original.fifa_ranking !== team.fifa_ranking;
        if (isDifferent) {
          newEditedIds.add(team.id);
        }
      }
    });

    set({
      teams: preset.teams.map(t => ({ ...t })),
      editedTeamIds: newEditedIds,
      teamsModified: newEditedIds.size > 0,
      activePresetName: name,
    });

    // Update current edits storage to match loaded preset
    const { saveCurrentEditsToStorage } = get();
    saveCurrentEditsToStorage();
  },

  deletePreset: (name: string) => {
    const { presets, activePresetName } = get();
    const newPresets = presets.filter(p => p.name !== name);

    set({
      presets: newPresets,
      activePresetName: activePresetName === name ? null : activePresetName,
    });

    try {
      localStorage.setItem(STORAGE_KEY_PRESETS, JSON.stringify(newPresets));
    } catch (e) {
      console.warn('Failed to save presets to LocalStorage:', e);
    }
  },
}));

// Selector hooks for common data
export const useTeams = () => useSimulatorStore((state) => state.teams);
export const useGroups = () => useSimulatorStore((state) => state.groups);
export const useResults = () => useSimulatorStore((state) => state.results);
export const useIsSimulating = () => useSimulatorStore((state) => state.isSimulating);
