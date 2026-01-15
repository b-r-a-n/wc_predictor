import { useMemo } from 'react';
import { useSimulatorStore } from '../../store/simulatorStore';
import { getFlagEmoji, formatPercent } from '../../utils/formatting';
import type { Team, TeamStatistics } from '../../types';

interface RoundTeam {
  team: Team;
  stats: TeamStatistics;
  probability: number;
}

export function KnockoutBracket() {
  const { results, teams } = useSimulatorStore();

  const teamMap = useMemo(() => {
    const map = new Map<number, Team>();
    teams.forEach((t) => map.set(t.id, t));
    return map;
  }, [teams]);

  // Get top teams for each round based on probabilities
  const roundData = useMemo(() => {
    if (!results) return null;

    const getTopTeams = (
      roundKey: keyof TeamStatistics,
      count: number
    ): RoundTeam[] => {
      const teamsWithStats = Object.values(results.team_stats)
        .map((stats) => ({
          team: teamMap.get(stats.team_id)!,
          stats,
          probability: (stats[roundKey] as number) / results.total_simulations,
        }))
        .filter((t) => t.team && t.probability > 0)
        .sort((a, b) => b.probability - a.probability)
        .slice(0, count);

      return teamsWithStats;
    };

    return {
      r32: getTopTeams('reached_round_of_32', 32),
      r16: getTopTeams('reached_round_of_16', 16),
      qf: getTopTeams('reached_quarter_finals', 8),
      sf: getTopTeams('reached_semi_finals', 4),
      final: getTopTeams('reached_final', 2),
      champion: getTopTeams('champion', 1),
    };
  }, [results, teamMap]);

  if (!results || !roundData) {
    return (
      <div className="flex items-center justify-center h-64 text-gray-500">
        Run a simulation to see the knockout bracket
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h2 className="text-lg font-semibold text-gray-900">Knockout Stage Probabilities</h2>
        <span className="text-sm text-gray-500">
          Most likely teams at each round
        </span>
      </div>

      {/* Champion */}
      <div className="bg-gradient-to-r from-yellow-400 to-yellow-600 rounded-lg p-6 text-center">
        <div className="text-yellow-100 text-sm font-medium mb-2">CHAMPION</div>
        {roundData.champion[0] && (
          <div className="flex flex-col items-center">
            <span className="text-4xl mb-2">
              {getFlagEmoji(roundData.champion[0].team.code)}
            </span>
            <span className="text-white text-xl font-bold">
              {roundData.champion[0].team.name}
            </span>
            <span className="text-yellow-100 text-lg">
              {formatPercent(roundData.champion[0].probability)}
            </span>
          </div>
        )}
      </div>

      {/* Final */}
      <RoundSection title="Final" teams={roundData.final} columns={2} />

      {/* Semi-finals */}
      <RoundSection title="Semi-Finals" teams={roundData.sf} columns={4} />

      {/* Quarter-finals */}
      <RoundSection title="Quarter-Finals" teams={roundData.qf} columns={4} />

      {/* Round of 16 */}
      <RoundSection title="Round of 16" teams={roundData.r16} columns={4} />

      {/* Round of 32 - collapsed by default on mobile */}
      <details className="group">
        <summary className="cursor-pointer list-none">
          <div className="flex items-center gap-2 text-gray-700 hover:text-gray-900">
            <span className="text-sm font-medium">Round of 32</span>
            <span className="text-xs text-gray-500">({roundData.r32.length} teams)</span>
            <span className="group-open:rotate-180 transition-transform">&#9660;</span>
          </div>
        </summary>
        <div className="mt-3">
          <RoundSection teams={roundData.r32} columns={4} />
        </div>
      </details>
    </div>
  );
}

interface RoundSectionProps {
  title?: string;
  teams: RoundTeam[];
  columns: number;
}

function RoundSection({ title, teams, columns }: RoundSectionProps) {
  const gridCols = {
    2: 'grid-cols-2',
    4: 'grid-cols-2 md:grid-cols-4',
    8: 'grid-cols-2 md:grid-cols-4 lg:grid-cols-8',
  }[columns] || 'grid-cols-4';

  return (
    <div className="space-y-2">
      {title && (
        <h3 className="text-sm font-medium text-gray-700">{title}</h3>
      )}
      <div className={`grid ${gridCols} gap-2`}>
        {teams.map((t) => (
          <TeamCard key={t.team.id} team={t.team} probability={t.probability} />
        ))}
      </div>
    </div>
  );
}

interface TeamCardProps {
  team: Team;
  probability: number;
}

function TeamCard({ team, probability }: TeamCardProps) {
  // Color based on probability
  const bgColor =
    probability > 0.8
      ? 'bg-green-100 border-green-300'
      : probability > 0.5
        ? 'bg-blue-50 border-blue-200'
        : probability > 0.3
          ? 'bg-gray-50 border-gray-200'
          : 'bg-white border-gray-100';

  return (
    <div className={`${bgColor} rounded border p-2 flex items-center gap-2`}>
      <span className="text-lg">{getFlagEmoji(team.code)}</span>
      <div className="flex-1 min-w-0">
        <div className="text-sm font-medium text-gray-900 truncate">{team.name}</div>
        <div className="text-xs text-gray-500">{formatPercent(probability)}</div>
      </div>
    </div>
  );
}
