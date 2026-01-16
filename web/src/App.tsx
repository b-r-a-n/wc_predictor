import { useEffect } from 'react';
import { Layout } from './components/layout';
import { WinProbabilityTable } from './components/results';
import { GroupStageView } from './components/groups';
import { KnockoutBracket } from './components/bracket';
import { HeadToHeadCalculator } from './components/calculator';
import { TeamDataEditor } from './components/editor';
import { LoadingSpinner } from './components/common';
import { useWasm } from './hooks/useWasm';
import { useSimulatorStore } from './store/simulatorStore';

function App() {
  const { status, api, error } = useWasm();
  const { setWasmStatus, setWasmApi, activeTab } = useSimulatorStore();

  useEffect(() => {
    if (status === 'loading') {
      setWasmStatus('loading');
    } else if (status === 'error') {
      setWasmStatus('error', error);
    } else if (status === 'ready' && api) {
      setWasmApi(api);
    }
  }, [status, api, error, setWasmStatus, setWasmApi]);

  if (status === 'loading') {
    return (
      <div className="min-h-screen bg-gray-100 flex items-center justify-center">
        <div className="text-center">
          <LoadingSpinner size="lg" className="mx-auto mb-4" />
          <p className="text-gray-600">Loading simulator...</p>
        </div>
      </div>
    );
  }

  if (status === 'error') {
    return (
      <div className="min-h-screen bg-gray-100 flex items-center justify-center">
        <div className="text-center max-w-md p-6 bg-white rounded-lg shadow">
          <div className="text-red-500 text-4xl mb-4">!</div>
          <h2 className="text-xl font-semibold text-gray-900 mb-2">Failed to Load</h2>
          <p className="text-gray-600 mb-4">{error || 'Unknown error occurred'}</p>
          <button
            onClick={() => window.location.reload()}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
          >
            Retry
          </button>
        </div>
      </div>
    );
  }

  return (
    <Layout>
      {activeTab === 'results' && <WinProbabilityTable />}
      {activeTab === 'groups' && <GroupStageView />}
      {activeTab === 'bracket' && <KnockoutBracket />}
      {activeTab === 'calculator' && <HeadToHeadCalculator />}
      {activeTab === 'editor' && <TeamDataEditor />}
    </Layout>
  );
}

export default App;
