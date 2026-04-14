import type { ScheduledMatch, Team, AggregatedResults, Group } from '../../types';
import { getFlagEmoji } from '../../utils/formatting';
import { getRoundDisplayName, formatMatchDate, formatMatchTime, resolveKnockoutMatchMatrix } from '../../utils/venueUtils';
import { getSlotForMatch } from '../../utils/matchMapping';
import { VenueMatchupMatrix } from './VenueMatchupMatrix';

interface VenueMatchCardProps {
  match: ScheduledMatch;
  teams: Team[];
  groups: Group[];
  results: AggregatedResults | null;
}

export function VenueMatchCard({ match, teams, groups, results }: VenueMatchCardProps) {
  const isGroupStage = match.round === 'group_stage';

  // For group stage, resolve teams from placeholders like "A1", "A2"
  const getGroupTeam = (placeholder: string | undefined): Team | null => {
    if (!placeholder || !match.groupId) return null;

    // Extract position from placeholder (e.g., "A1" -> 1, "A2" -> 2)
    const posMatch = placeholder.match(/^([A-L])(\d)$/);
    if (!posMatch) return null;

    const groupId = posMatch[1];
    const position = parseInt(posMatch[2], 10);

    // Find the group
    const group = groups.find(g => g.id === groupId);
    if (!group) return null;

    // Get team at position (1-indexed in placeholder, 0-indexed in array)
    const teamId = group.teams[position - 1];
    return teams.find(t => t.id === teamId) || null;
  };

  // Render a team with flag and name
  const renderTeam = (team: Team) => (
    <div className="flex items-center gap-2">
      <span className="text-lg">{getFlagEmoji(team.code)}</span>
      <span className="font-medium text-gray-900">{team.name}</span>
    </div>
  );

  // Get match participants
  let matchContent: React.ReactNode;

  if (isGroupStage) {
    const homeTeam = getGroupTeam(match.homePlaceholder);
    const awayTeam = getGroupTeam(match.awayPlaceholder);

    const homeContent = homeTeam ? renderTeam(homeTeam) : (
      <span className="text-gray-400">{match.homePlaceholder || 'TBD'}</span>
    );
    const awayContent = awayTeam ? renderTeam(awayTeam) : (
      <span className="text-gray-400">{match.awayPlaceholder || 'TBD'}</span>
    );

    matchContent = (
      <div className="flex items-center justify-between">
        <div className="flex-1 min-w-0">
          {homeContent}
        </div>
        <div className="px-4 text-gray-400 font-semibold">
          vs
        </div>
        <div className="flex-1 min-w-0 text-right">
          <div className="flex justify-end">
            {awayContent}
          </div>
        </div>
      </div>
    );
  } else {
    // Knockout match - show matchup probability matrix from simulation results
    const knockoutSlot = getSlotForMatch(match.matchNumber) ?? 0;

    const matrix = resolveKnockoutMatchMatrix(
      match.round as Exclude<typeof match.round, 'group_stage'>,
      knockoutSlot,
      results,
      teams
    );

    matchContent = matrix ? (
      <VenueMatchupMatrix data={matrix} />
    ) : (
      <div className="text-gray-400 text-sm italic text-center">
        Run simulation to see predicted matchups
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-4">
      {/* Match header */}
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-3">
          <span className="text-xs font-medium text-gray-500 bg-gray-100 px-2 py-1 rounded">
            #{match.matchNumber}
          </span>
          <span className="text-sm font-medium text-blue-600">
            {getRoundDisplayName(match.round)}
          </span>
          {match.groupId && (
            <span className="text-sm text-gray-500">
              Group {match.groupId}
            </span>
          )}
        </div>
        <div className="text-right text-sm text-gray-600">
          <div>{formatMatchDate(match.date)}</div>
          <div className="text-xs text-gray-400">{formatMatchTime(match.time)}</div>
        </div>
      </div>

      {/* Match participants */}
      {matchContent}
    </div>
  );
}
