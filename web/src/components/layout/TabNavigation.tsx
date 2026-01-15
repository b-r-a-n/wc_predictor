import { useSimulatorStore } from '../../store/simulatorStore';
import type { TabId } from '../../types';

const tabs: { id: TabId; label: string; icon: string }[] = [
  { id: 'results', label: 'Results', icon: '\uD83C\uDFC6' },
  { id: 'groups', label: 'Groups', icon: '\uD83D\uDCCA' },
  { id: 'bracket', label: 'Bracket', icon: '\uD83C\uDF33' },
  { id: 'calculator', label: 'Calculator', icon: '\uD83E\uDDEE' },
];

export function TabNavigation() {
  const { activeTab, setActiveTab } = useSimulatorStore();

  return (
    <nav className="bg-white border-b border-gray-200">
      <div className="flex overflow-x-auto">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`flex items-center gap-2 px-6 py-3 text-sm font-medium whitespace-nowrap border-b-2 transition-colors ${
              activeTab === tab.id
                ? 'border-blue-600 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            }`}
          >
            <span>{tab.icon}</span>
            <span>{tab.label}</span>
          </button>
        ))}
      </div>
    </nav>
  );
}
