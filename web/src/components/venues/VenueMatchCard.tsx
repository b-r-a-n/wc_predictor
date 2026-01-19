import type { ScheduledMatch, Team, TeamProbability, AggregatedResults, Group } from '../../types';
import { getFlagEmoji, formatPercent } from '../../utils/formatting';
import { getRoundDisplayName, formatMatchDate, formatMatchTime, resolveKnockoutMatchCandidates } from '../../utils/venueUtils';

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

  // Render probability candidates for knockout matches
  const renderCandidates = (candidates: TeamProbability[], label: string) => {
    if (candidates.length === 0) {
      return (
        <div className="text-gray-400 text-sm italic">
          {label}
        </div>
      );
    }

    return (
      <div className="space-y-1">
        {candidates.slice(0, 3).map(({ team, probability }) => (
          <div key={team.id} className="flex items-center gap-2 text-sm">
            <span>{getFlagEmoji(team.code)}</span>
            <span className="text-gray-700">{team.name}</span>
            <span className="text-gray-400">({formatPercent(probability, 0)})</span>
          </div>
        ))}
        {candidates.length > 3 && (
          <div className="text-xs text-gray-400">
            +{candidates.length - 3} more
          </div>
        )}
      </div>
    );
  };

  // Get match participants
  let homeContent: React.ReactNode;
  let awayContent: React.ReactNode;

  if (isGroupStage) {
    const homeTeam = getGroupTeam(match.homePlaceholder);
    const awayTeam = getGroupTeam(match.awayPlaceholder);

    homeContent = homeTeam ? renderTeam(homeTeam) : (
      <span className="text-gray-400">{match.homePlaceholder || 'TBD'}</span>
    );
    awayContent = awayTeam ? renderTeam(awayTeam) : (
      <span className="text-gray-400">{match.awayPlaceholder || 'TBD'}</span>
    );
  } else {
    // Knockout match - show candidates if simulation results available
    const knockoutSlot = match.knockoutSlot ?? 0;

    // For knockout, we need to determine which teams could be in each slot
    // The slot represents a bracket position, and we need candidates for both home and away
    // Home slot is knockoutSlot * 2, away slot is knockoutSlot * 2 + 1 for earlier rounds
    // But actually, the bracket_slot_stats tracks teams per slot per round directly

    // Get candidates for this specific knockout position
    const homeCandidates = resolveKnockoutMatchCandidates(
      match.round as Exclude<typeof match.round, 'group_stage'>,
      knockoutSlot,
      results,
      teams
    );

    // For the same match, both home and away are drawn from the same pool
    // The bracket structure means opponent tracking is complex
    // For simplicity, we show the same candidates for both sides initially
    // In practice, the actual matchups depend on who wins previous rounds

    if (homeCandidates.length > 0) {
      // Show top candidates
      homeContent = renderCandidates(homeCandidates.slice(0, 3), match.homePlaceholder || 'TBD');
      // For away, show remaining candidates or placeholder
      const awayCandidates = homeCandidates.slice(3, 6);
      awayContent = awayCandidates.length > 0
        ? renderCandidates(awayCandidates, match.awayPlaceholder || 'TBD')
        : <span className="text-gray-400 text-sm italic">{match.awayPlaceholder || 'TBD'}</span>;
    } else {
      homeContent = <span className="text-gray-400 text-sm italic">{match.homePlaceholder || 'TBD'}</span>;
      awayContent = <span className="text-gray-400 text-sm italic">{match.awayPlaceholder || 'TBD'}</span>;
    }
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
    </div>
  );
}
