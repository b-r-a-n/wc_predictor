import { useSimulatorStore } from '../../store/simulatorStore';

export function Header() {
  const { wasmStatus, wasmApi } = useSimulatorStore();

  return (
    <header className="bg-gradient-to-r from-blue-900 to-blue-700 text-white shadow-lg">
      <div className="max-w-7xl mx-auto px-4 py-4">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold">World Cup 2026 Simulator</h1>
            <p className="text-blue-200 text-sm">Monte Carlo Prediction Engine</p>
          </div>
          <div className="text-right">
            <div className="flex items-center gap-2">
              <span
                className={`w-2 h-2 rounded-full ${
                  wasmStatus === 'ready'
                    ? 'bg-green-400'
                    : wasmStatus === 'loading'
                      ? 'bg-yellow-400 animate-pulse'
                      : 'bg-red-400'
                }`}
              />
              <span className="text-sm text-blue-200">
                {wasmStatus === 'ready'
                  ? 'Ready'
                  : wasmStatus === 'loading'
                    ? 'Loading...'
                    : 'Error'}
              </span>
            </div>
            {wasmApi && (
              <span className="text-xs text-blue-300">v{wasmApi.version}</span>
            )}
          </div>
        </div>
      </div>
    </header>
  );
}
