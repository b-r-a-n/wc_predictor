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
  const [isPanelOpen, setIsPanelOpen] = useState(false);

  return (
    <div className="min-h-screen bg-gray-100 flex flex-col">
      <Header onOpenSettings={() => setIsPanelOpen(true)} />
      <TabNavigation />

      <main className="flex-1 p-4 lg:p-6 overflow-auto">{children}</main>

      {isPanelOpen && (
        <>
          <div
            className="fixed inset-0 bg-black/30 z-40"
            onClick={() => setIsPanelOpen(false)}
          />
          <div className="fixed top-0 right-0 h-full w-full sm:w-80 bg-white shadow-xl z-50 overflow-y-auto">
            <div className="p-4">
              <div className="flex justify-between items-center mb-4">
                <h2 className="text-lg font-semibold">Simulation Settings</h2>
                <button
                  onClick={() => setIsPanelOpen(false)}
                  className="text-gray-500 hover:text-gray-700 text-2xl leading-none"
                  aria-label="Close"
                >
                  &times;
                </button>
              </div>
              <ControlPanel hideHeader />
            </div>
          </div>
        </>
      )}

      <LoadingModal isOpen={isSimulating} />
    </div>
  );
}
