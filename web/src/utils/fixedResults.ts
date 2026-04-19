import type {
  Group,
  MatchScheduleData,
  ScheduledMatch,
  FixedMatchResult,
  RustFixedMatchEntry,
} from '../types';

function groupPositionFromPlaceholder(placeholder?: string): number | null {
  if (!placeholder) return null;
  const digit = placeholder.slice(1);
  const n = parseInt(digit, 10);
  if (!Number.isFinite(n) || n < 1 || n > 4) return null;
  return n - 1;
}

export function resolveGroupMatchTeams(
  match: ScheduledMatch,
  groups: Group[]
): { groupId: string; homeTeamId: number; awayTeamId: number } | null {
  if (match.round !== 'group_stage' || !match.groupId) return null;
  const group = groups.find((g) => g.id === match.groupId);
  if (!group) return null;

  // If explicit team IDs are provided, trust them.
  if (match.homeTeamId != null && match.awayTeamId != null) {
    return {
      groupId: group.id,
      homeTeamId: match.homeTeamId,
      awayTeamId: match.awayTeamId,
    };
  }

  const homeIdx = groupPositionFromPlaceholder(match.homePlaceholder);
  const awayIdx = groupPositionFromPlaceholder(match.awayPlaceholder);
  if (homeIdx == null || awayIdx == null) return null;

  return {
    groupId: group.id,
    homeTeamId: group.teams[homeIdx],
    awayTeamId: group.teams[awayIdx],
  };
}

/**
 * Convert the app's fixed-results map (keyed by scheduled match number) into
 * the array format that the Rust `FixedResults` type deserializes from.
 *
 * Group-stage fixtures in the simulator are canonically ordered by the team's
 * position within its group (teams[i] vs teams[j] with i < j). The FIFA
 * schedule sometimes swaps home/away (e.g. D4 vs D2). We canonicalize the
 * entry here so the Rust lookup matches — swapping the score along with the
 * team order preserves the user's intent.
 *
 * Silently drops matches whose teams we can't resolve.
 */
export function toRustFixedResults(
  fixedResults: Record<number, FixedMatchResult>,
  schedule: MatchScheduleData,
  groups: Group[]
): RustFixedMatchEntry[] {
  const entries: RustFixedMatchEntry[] = [];
  for (const result of Object.values(fixedResults)) {
    const match = schedule.matches.find((m) => m.matchNumber === result.matchNumber);
    if (!match) continue;
    const resolved = resolveGroupMatchTeams(match, groups);
    if (!resolved) continue;

    const group = groups.find((g) => g.id === resolved.groupId);
    if (!group) continue;

    let homeTeam = resolved.homeTeamId;
    let awayTeam = resolved.awayTeamId;
    let homeGoals = result.homeScore;
    let awayGoals = result.awayScore;

    const iHome = group.teams.indexOf(homeTeam);
    const iAway = group.teams.indexOf(awayTeam);
    if (iHome > iAway) {
      [homeTeam, awayTeam] = [awayTeam, homeTeam];
      [homeGoals, awayGoals] = [awayGoals, homeGoals];
    }

    entries.push({
      fixture: {
        type: 'GroupStage',
        group_id: resolved.groupId,
        home_team: homeTeam,
        away_team: awayTeam,
      },
      spec: {
        mode: 'ExactScore',
        home_goals: homeGoals,
        away_goals: awayGoals,
      },
    });
  }
  return entries;
}
