import { useSimulatorStore } from '../../store/simulatorStore';
import { GroupTable } from './GroupTable';

export function GroupStageView() {
  const { groups, teams, results } = useSimulatorStore();

  if (!results) {
    return (
      <div className="flex items-center justify-center h-64 text-gray-500">
        Run a simulation to see group stage results
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <h2 className="text-lg font-semibold text-gray-900">Group Stage Probabilities</h2>
        <span className="text-sm text-gray-500">
          Based on {results.total_simulations.toLocaleString()} simulations
        </span>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
        {groups.map((group) => (
          <GroupTable
            key={group.id}
            group={group}
            teams={teams}
            teamStats={results.team_stats}
            totalSimulations={results.total_simulations}
          />
        ))}
      </div>
    </div>
  );
}
