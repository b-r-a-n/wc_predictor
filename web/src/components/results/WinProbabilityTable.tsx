import { useMemo, useState } from 'react';
import { useSimulatorStore } from '../../store/simulatorStore';
import { getFlagEmoji, formatPercent } from '../../utils/formatting';
// Types are inferred from the store

type SortKey = 'rank' | 'name' | 'win' | 'final' | 'semi' | 'knockout';
type SortDirection = 'asc' | 'desc';

export function WinProbabilityTable() {
  const { results, teams } = useSimulatorStore();
  const [sortKey, setSortKey] = useState<SortKey>('win');
  const [sortDirection, setSortDirection] = useState<SortDirection>('desc');

  const teamMap = useMemo(() => {
    const map = new Map<number, (typeof teams)[0]>();
    teams.forEach((t) => map.set(t.id, t));
    return map;
  }, [teams]);

  const sortedStats = useMemo(() => {
    if (!results) return [];

    const stats = Object.values(results.team_stats);

    return stats.sort((a, b) => {
      const team_a = teamMap.get(a.team_id);
      const team_b = teamMap.get(b.team_id);
      let comparison = 0;

      switch (sortKey) {
        case 'name':
          comparison = (team_a?.name ?? '').localeCompare(team_b?.name ?? '');
          break;
        case 'win':
          comparison = a.win_probability - b.win_probability;
          break;
        case 'final':
          comparison = a.final_probability - b.final_probability;
          break;
        case 'semi':
          comparison = a.semi_final_probability - b.semi_final_probability;
          break;
        case 'knockout':
          comparison = a.knockout_probability - b.knockout_probability;
          break;
        default:
          comparison = a.win_probability - b.win_probability;
      }

      return sortDirection === 'asc' ? comparison : -comparison;
    });
  }, [results, sortKey, sortDirection, teamMap]);

  const handleSort = (key: SortKey) => {
    if (key === sortKey) {
      setSortDirection((d) => (d === 'asc' ? 'desc' : 'asc'));
    } else {
      setSortKey(key);
      setSortDirection('desc');
    }
  };

  const SortHeader = ({
    column,
    label,
    className = '',
  }: {
    column: SortKey;
    label: string;
    className?: string;
  }) => (
    <th
      className={`px-3 py-2 text-left text-xs font-medium text-gray-500 uppercase tracking-wider cursor-pointer hover:bg-gray-100 ${className}`}
      onClick={() => handleSort(column)}
    >
      <div className="flex items-center gap-1">
        {label}
        {sortKey === column && <span>{sortDirection === 'asc' ? '\u25B2' : '\u25BC'}</span>}
      </div>
    </th>
  );

  if (!results) {
    return (
      <div className="flex items-center justify-center h-64 text-gray-500">
        Run a simulation to see results
      </div>
    );
  }

  return (
    <div className="overflow-x-auto">
      <table className="min-w-full divide-y divide-gray-200">
        <thead className="bg-gray-50">
          <tr>
            <SortHeader column="rank" label="#" className="w-12" />
            <SortHeader column="name" label="Team" />
            <SortHeader column="win" label="Win %" />
            <SortHeader column="final" label="Final %" />
            <SortHeader column="semi" label="Semi %" />
            <SortHeader column="knockout" label="R32 %" />
          </tr>
        </thead>
        <tbody className="bg-white divide-y divide-gray-200">
          {sortedStats.map((stats, index) => {
            const team = teamMap.get(stats.team_id);
            const isTopTeam = index < 8 && sortKey === 'win' && sortDirection === 'desc';

            return (
              <tr
                key={stats.team_id}
                className={isTopTeam ? 'bg-yellow-50' : index % 2 === 0 ? 'bg-white' : 'bg-gray-50'}
              >
                <td className="px-3 py-2 whitespace-nowrap text-sm text-gray-500">{index + 1}</td>
                <td className="px-3 py-2 whitespace-nowrap">
                  <div className="flex items-center gap-2">
                    <span className="text-lg">{team ? getFlagEmoji(team.code) : ''}</span>
                    <span className="text-sm font-medium text-gray-900">{stats.team_name}</span>
                  </div>
                </td>
                <td className="px-3 py-2 whitespace-nowrap">
                  <ProbabilityCell value={stats.win_probability} color="bg-green-500" />
                </td>
                <td className="px-3 py-2 whitespace-nowrap">
                  <ProbabilityCell value={stats.final_probability} color="bg-blue-500" />
                </td>
                <td className="px-3 py-2 whitespace-nowrap">
                  <ProbabilityCell value={stats.semi_final_probability} color="bg-purple-500" />
                </td>
                <td className="px-3 py-2 whitespace-nowrap">
                  <ProbabilityCell value={stats.knockout_probability} color="bg-orange-500" />
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

function ProbabilityCell({ value, color }: { value: number; color: string }) {
  const percentage = value * 100;
  return (
    <div className="relative w-24">
      <div className="absolute inset-0 h-full bg-gray-100 rounded">
        <div
          className={`h-full ${color} rounded opacity-30`}
          style={{ width: `${Math.min(100, percentage)}%` }}
        />
      </div>
      <span className="relative z-10 text-sm font-medium text-gray-900 px-2">
        {formatPercent(value)}
      </span>
    </div>
  );
}
