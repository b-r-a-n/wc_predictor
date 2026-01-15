import { Button } from '../common';
import { StrategySelector } from './StrategySelector';
import { CompositeWeights } from './CompositeWeights';
import { IterationSlider } from './IterationSlider';
import { useSimulatorStore } from '../../store/simulatorStore';

export function ControlPanel() {
  const {
    strategy,
    setStrategy,
    iterations,
    setIterations,
    compositeWeights,
    setCompositeWeights,
    runSimulation,
    isSimulating,
    wasmStatus,
  } = useSimulatorStore();

  const isReady = wasmStatus === 'ready' && !isSimulating;

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-semibold text-gray-900 mb-4">Simulation Settings</h2>
      </div>

      <StrategySelector value={strategy} onChange={setStrategy} />

      {strategy === 'composite' && (
        <CompositeWeights value={compositeWeights} onChange={setCompositeWeights} />
      )}

      <IterationSlider value={iterations} onChange={setIterations} />

      <Button onClick={runSimulation} disabled={!isReady} className="w-full" size="lg">
        {isSimulating ? 'Running...' : 'Run Simulation'}
      </Button>
    </div>
  );
}
