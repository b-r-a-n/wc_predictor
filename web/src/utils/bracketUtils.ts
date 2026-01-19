import type { Team, AggregatedResults, MostLikelyBracket, MostLikelySlotData, BracketSlotStats } from '../types';

type RoundKey = 'round_of_32' | 'round_of_16' | 'quarter_finals' | 'semi_finals';

/**
 * Compute the most likely bracket by finding the most probable team for each slot.
 * Aggregates bracket_slot_stats across all teams to find winners.
 */
export function computeMostLikelyBracket(
  results: AggregatedResults,
  teamMap: Map<number, Team>
): MostLikelyBracket | null {
  const bracketSlotStats = results.bracket_slot_stats;
  if (!bracketSlotStats) return null;

  const totalSimulations = results.total_simulations;

  // Helper to find the most likely team for a specific slot in a round
  const findMostLikelyForSlot = (
    roundKey: RoundKey,
    slotIndex: string
  ): MostLikelySlotData | null => {
    let bestTeamId: number | null = null;
    let bestCount = 0;

    // Iterate through all teams' bracket stats
    for (const [teamIdStr, stats] of Object.entries(bracketSlotStats)) {
      const teamStats = stats as BracketSlotStats;
      const roundData = teamStats[roundKey] as Record<string, number> | undefined;
      if (!roundData) continue;

      const count = roundData[slotIndex] || 0;
      if (count > bestCount) {
        bestCount = count;
        bestTeamId = parseInt(teamIdStr);
      }
    }

    if (bestTeamId === null || bestCount === 0) return null;

    const team = teamMap.get(bestTeamId);
    if (!team) return null;

    return {
      teamId: bestTeamId,
      team,
      count: bestCount,
      probability: bestCount / totalSimulations,
    };
  };

  // Helper to find the most likely team for the final slot
  const findMostLikelyForFinal = (): MostLikelySlotData | null => {
    let bestTeamId: number | null = null;
    let bestCount = 0;

    for (const [teamIdStr, stats] of Object.entries(bracketSlotStats)) {
      const teamStats = stats as BracketSlotStats;
      const count = teamStats.final_match || 0;
      if (count > bestCount) {
        bestCount = count;
        bestTeamId = parseInt(teamIdStr);
      }
    }

    if (bestTeamId === null || bestCount === 0) return null;

    const team = teamMap.get(bestTeamId);
    if (!team) return null;

    return {
      teamId: bestTeamId,
      team,
      count: bestCount,
      probability: bestCount / totalSimulations,
    };
  };

  // Build the most likely bracket
  const bracket: MostLikelyBracket = {
    round_of_32: {},
    round_of_16: {},
    quarter_finals: {},
    semi_finals: {},
    final_match: null,
    champion: null,
  };

  // R32: 16 slots (0-15)
  for (let i = 0; i < 16; i++) {
    const slotData = findMostLikelyForSlot('round_of_32', String(i));
    if (slotData) {
      bracket.round_of_32[String(i)] = slotData;
    }
  }

  // R16: 8 slots (0-7)
  for (let i = 0; i < 8; i++) {
    const slotData = findMostLikelyForSlot('round_of_16', String(i));
    if (slotData) {
      bracket.round_of_16[String(i)] = slotData;
    }
  }

  // QF: 4 slots (0-3)
  for (let i = 0; i < 4; i++) {
    const slotData = findMostLikelyForSlot('quarter_finals', String(i));
    if (slotData) {
      bracket.quarter_finals[String(i)] = slotData;
    }
  }

  // SF: 2 slots (0-1)
  for (let i = 0; i < 2; i++) {
    const slotData = findMostLikelyForSlot('semi_finals', String(i));
    if (slotData) {
      bracket.semi_finals[String(i)] = slotData;
    }
  }

  // Final
  bracket.final_match = findMostLikelyForFinal();

  // Champion is the most_likely_winner from results
  const championTeam = teamMap.get(results.most_likely_winner);
  if (championTeam) {
    const championStats = bracketSlotStats[String(results.most_likely_winner)] as BracketSlotStats | undefined;
    const championCount = championStats?.final_match || 0;
    bracket.champion = {
      teamId: results.most_likely_winner,
      team: championTeam,
      count: championCount,
      probability: championCount / totalSimulations,
    };
  }

  return bracket;
}

/**
 * Compute the set of slot keys where the predicted champion appears.
 * This is used to highlight the winner's predicted path through the bracket.
 */
export function computeWinnerPathHighlights(
  bracket: MostLikelyBracket
): Set<string> {
  const highlights = new Set<string>();

  if (!bracket.champion) return highlights;

  const championId = bracket.champion.teamId;

  // Check each round for the champion
  const rounds: { key: keyof MostLikelyBracket; roundIndex: number }[] = [
    { key: 'round_of_32', roundIndex: 0 },
    { key: 'round_of_16', roundIndex: 1 },
    { key: 'quarter_finals', roundIndex: 2 },
    { key: 'semi_finals', roundIndex: 3 },
  ];

  for (const { key, roundIndex } of rounds) {
    const roundData = bracket[key] as Record<string, MostLikelySlotData>;
    for (const [slotStr, slotData] of Object.entries(roundData)) {
      if (slotData.teamId === championId) {
        highlights.add(`${roundIndex}-${slotStr}`);
      }
    }
  }

  // Add final if champion is predicted
  if (bracket.final_match && bracket.final_match.teamId === championId) {
    highlights.add('4-0');
  }

  return highlights;
}
