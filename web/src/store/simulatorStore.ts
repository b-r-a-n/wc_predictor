import { create } from 'zustand';
import type { Team, Group, AggregatedResults, Strategy, CompositeWeights, TabId, WasmStatus } from '../types';
import type { WasmApi } from '../hooks/useWasm';

interface SimulatorState {
  // WASM state
  wasmStatus: WasmStatus;
  wasmError: string | null;
  wasmApi: WasmApi | null;
  teams: Team[];
  groups: Group[];

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
}

export const useSimulatorStore = create<SimulatorState>((set, get) => ({
  // Initial WASM state
  wasmStatus: 'loading',
  wasmError: null,
  wasmApi: null,
  teams: [],
  groups: [],

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

  setWasmApi: (api) =>
    set({
      wasmApi: api,
      teams: api.teams,
      groups: api.groups,
      wasmStatus: 'ready',
    }),

  setStrategy: (strategy) => set({ strategy }),

  setIterations: (iterations) => set({ iterations }),

  setCompositeWeights: (weights) => set({ compositeWeights: weights }),

  setActiveTab: (tab) => set({ activeTab: tab }),

  runSimulation: () => {
    const { wasmApi, strategy, iterations, compositeWeights, isSimulating } = get();

    if (!wasmApi || isSimulating) return;

    set({ isSimulating: true });

    // Use setTimeout to allow UI to update before blocking simulation
    setTimeout(() => {
      try {
        const results = wasmApi.runSimulation(strategy, iterations, compositeWeights);
        set({ results, isSimulating: false });
      } catch (error) {
        console.error('Simulation error:', error);
        set({ isSimulating: false });
      }
    }, 50);
  },
}));

// Selector hooks for common data
export const useTeams = () => useSimulatorStore((state) => state.teams);
export const useGroups = () => useSimulatorStore((state) => state.groups);
export const useResults = () => useSimulatorStore((state) => state.results);
export const useIsSimulating = () => useSimulatorStore((state) => state.isSimulating);
