import { useState } from 'react';
import type { ReactNode } from 'react';
import { Header } from './Header';
import { TabNavigation } from './TabNavigation';
import { ControlPanel } from '../controls/ControlPanel';
import { LoadingModal } from '../common/LoadingModal';
import { useSimulatorStore } from '../../store/simulatorStore';

interface LayoutProps {
  children: ReactNode;
}

export function Layout({ children }: LayoutProps) {
  const { isSimulating } = useSimulatorStore();
  const [isSidebarOpen, setIsSidebarOpen] = useState(false);

  return (
    <div className="min-h-screen bg-gray-100 flex flex-col">
      <Header />
      <TabNavigation />

      <div className="flex-1 flex flex-col lg:flex-row">
        {/* Desktop sidebar */}
        <aside className="hidden lg:block w-72 bg-white border-r border-gray-200 p-4 flex-shrink-0">
          <ControlPanel />
        </aside>

        {/* Mobile sidebar toggle */}
        <button
          className="lg:hidden fixed bottom-4 right-4 z-40 bg-blue-600 text-white p-4 rounded-full shadow-lg"
          onClick={() => setIsSidebarOpen(!isSidebarOpen)}
        >
          {isSidebarOpen ? '\u2715' : '\u2699'}
        </button>

        {/* Mobile sidebar overlay */}
        {isSidebarOpen && (
          <>
            <div
              className="lg:hidden fixed inset-0 bg-black/50 z-40"
              onClick={() => setIsSidebarOpen(false)}
            />
            <div className="lg:hidden fixed bottom-0 left-0 right-0 bg-white z-50 rounded-t-2xl shadow-xl max-h-[80vh] overflow-y-auto">
              <div className="p-4">
                <div className="flex justify-between items-center mb-4">
                  <h2 className="text-lg font-semibold">Settings</h2>
                  <button
                    onClick={() => setIsSidebarOpen(false)}
                    className="text-gray-500 hover:text-gray-700"
                  >
                    \u2715
                  </button>
                </div>
                <ControlPanel />
              </div>
            </div>
          </>
        )}

        {/* Main content */}
        <main className="flex-1 p-4 lg:p-6 overflow-auto">{children}</main>
      </div>

      <LoadingModal isOpen={isSimulating} />
    </div>
  );
}
