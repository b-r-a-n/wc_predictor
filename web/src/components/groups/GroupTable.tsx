import { useMemo } from 'react';
import type { Group, Team, TeamStatistics } from '../../types';
import { getFlagEmoji, formatPercent } from '../../utils/formatting';

interface GroupTableProps {
  group: Group;
  teams: Team[];
  teamStats: Record<number, TeamStatistics> | null;
  totalSimulations: number;
}

export function GroupTable({ group, teams, teamStats, totalSimulations }: GroupTableProps) {
  const groupTeams = useMemo(() => {
    return group.teams
      .map((id) => {
        const team = teams.find((t) => t.id === id);
        const stats = teamStats?.[id];
        return { team, stats };
      })
      .filter((t): t is { team: Team; stats: TeamStatistics | undefined } => t.team !== undefined)
      .sort((a, b) => {
        // Sort by knockout probability (descending)
        const aKnockout = a.stats?.knockout_probability ?? 0;
        const bKnockout = b.stats?.knockout_probability ?? 0;
        return bKnockout - aKnockout;
      });
  }, [group.teams, teams, teamStats]);

  return (
    <div className="bg-white rounded-lg shadow-md overflow-hidden">
      <div className="bg-gray-800 text-white px-4 py-2">
        <h3 className="font-semibold">Group {group.id}</h3>
      </div>
      <table className="min-w-full">
        <thead className="bg-gray-50">
          <tr>
            <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 uppercase">
              Team
            </th>
            <th className="px-2 py-2 text-center text-xs font-medium text-gray-500 uppercase w-16">
              Adv %
            </th>
            <th className="px-2 py-2 text-center text-xs font-medium text-gray-500 uppercase w-14">
              1st
            </th>
            <th className="px-2 py-2 text-center text-xs font-medium text-gray-500 uppercase w-14">
              2nd
            </th>
            <th className="px-2 py-2 text-center text-xs font-medium text-gray-500 uppercase w-14">
              3rd*
            </th>
          </tr>
        </thead>
        <tbody className="divide-y divide-gray-200">
          {groupTeams.map(({ team, stats }, index) => {
            const first =
              stats && totalSimulations > 0 ? stats.group_wins / totalSimulations : 0;
            const second =
              stats && totalSimulations > 0 ? stats.group_second / totalSimulations : 0;
            const thirdQ =
              stats && totalSimulations > 0
                ? stats.group_third_qualified / totalSimulations
                : 0;
            const advance = first + second + thirdQ;

            return (
              <tr
                key={team.id}
                className={index % 2 === 0 ? 'bg-white' : 'bg-gray-50'}
              >
                <td className="px-3 py-2 whitespace-nowrap">
                  <div className="flex items-center gap-2">
                    <span className="text-base">{getFlagEmoji(team.code)}</span>
                    <span className="text-sm font-medium text-gray-900">{team.name}</span>
                  </div>
                </td>
                <td className="px-2 py-2 text-center">
                  <span
                    className={`text-sm font-bold ${advance > 0.5 ? 'text-green-600' : 'text-gray-700'}`}
                  >
                    {formatPercent(advance, 0)}
                  </span>
                </td>
                <td className="px-2 py-2 text-center text-xs text-gray-600">
                  {formatPercent(first, 0)}
                </td>
                <td className="px-2 py-2 text-center text-xs text-gray-600">
                  {formatPercent(second, 0)}
                </td>
                <td className="px-2 py-2 text-center text-xs text-gray-600">
                  {formatPercent(thirdQ, 0)}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
      <div className="px-3 py-1 bg-gray-50 text-xs text-gray-500">
        * Best 8 third-placed teams qualify
      </div>
    </div>
  );
}
