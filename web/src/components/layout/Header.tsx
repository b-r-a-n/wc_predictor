import { useSimulatorStore } from '../../store/simulatorStore';

interface HeaderProps {
  onOpenSettings?: () => void;
}

export function Header({ onOpenSettings }: HeaderProps) {
  const { wasmStatus, wasmApi } = useSimulatorStore();

  return (
    <header className="bg-gradient-to-r from-blue-900 to-blue-700 text-white shadow-lg">
      <div className="max-w-7xl mx-auto px-4 py-4">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold">World Cup 2026 Simulator</h1>
            <p className="text-blue-200 text-sm">Monte Carlo Prediction Engine</p>
          </div>
          <div className="flex items-center gap-4">
            <div className="text-right">
              <div className="flex items-center justify-end gap-2">
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
              <div className="text-xs text-blue-300 space-x-2">
                {wasmApi && <span>sim v{wasmApi.version}</span>}
                <span>ui {__COMMIT_SHA__}</span>
              </div>
            </div>
            {onOpenSettings && (
              <button
                onClick={onOpenSettings}
                title="Simulation settings"
                aria-label="Open simulation settings"
                className="p-2 rounded-md text-blue-100 hover:bg-white/10 hover:text-white transition-colors"
              >
                <svg xmlns="http://www.w3.org/2000/svg" className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                  <path strokeLinecap="round" strokeLinejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                </svg>
              </button>
            )}
          </div>
        </div>
      </div>
    </header>
  );
}
