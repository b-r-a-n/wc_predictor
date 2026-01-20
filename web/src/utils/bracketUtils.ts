import type { Team, AggregatedResults, MostLikelySlotData, RustMostLikelyBracketSlot } from '../types';

// UI type for optimal R32 match with both teams resolved
export interface OptimalR32MatchDisplay {
  slot: number;
  teamA: MostLikelySlotData;
  teamB: MostLikelySlotData;
  winnerId: number;
  winner: Team;
}

// UI type for optimal bracket
export interface OptimalBracketDisplay {
  round_of_32: OptimalR32MatchDisplay[];
  round_of_16: Record<string, MostLikelySlotData>;
  quarter_finals: Record<string, MostLikelySlotData>;
  semi_finals: Record<string, MostLikelySlotData>;
  champion: MostLikelySlotData | null;
  jointProbability: number;
  logProbability: number;
}

/**
 * Convert the optimal bracket from Rust to UI format.
 * The Hungarian algorithm ensures exactly 32 unique teams in R32.
 */
export function computeOptimalBracket(
  results: AggregatedResults,
  teamMap: Map<number, Team>
): OptimalBracketDisplay | null {
  const rustBracket = results.optimal_bracket;
  if (!rustBracket) return null;

  // Helper to convert Rust slot data to UI format
  const convertSlot = (rustSlot: RustMostLikelyBracketSlot | undefined): MostLikelySlotData | null => {
    if (!rustSlot) return null;
    const team = teamMap.get(rustSlot.team_id);
    if (!team) return null;
    return {
      teamId: rustSlot.team_id,
      team,
      count: rustSlot.count,
      probability: rustSlot.probability,
    };
  };

  // Helper to convert a round's slots
  const convertRound = (
    rustRound: Record<string, RustMostLikelyBracketSlot> | undefined
  ): Record<string, MostLikelySlotData> => {
    const result: Record<string, MostLikelySlotData> = {};
    if (!rustRound) return result;
    for (const [slot, rustSlot] of Object.entries(rustRound)) {
      const slotData = convertSlot(rustSlot);
      if (slotData) {
        result[slot] = slotData;
      }
    }
    return result;
  };

  // Convert R32 matches
  const r32Matches: OptimalR32MatchDisplay[] = [];
  for (const rustMatch of rustBracket.round_of_32 || []) {
    const teamA = convertSlot(rustMatch.team_a);
    const teamB = convertSlot(rustMatch.team_b);
    const winner = teamMap.get(rustMatch.winner);

    if (teamA && teamB && winner) {
      r32Matches.push({
        slot: rustMatch.slot,
        teamA,
        teamB,
        winnerId: rustMatch.winner,
        winner,
      });
    }
  }

  // Sort by slot number
  r32Matches.sort((a, b) => a.slot - b.slot);

  const bracket: OptimalBracketDisplay = {
    round_of_32: r32Matches,
    round_of_16: convertRound(rustBracket.round_of_16),
    quarter_finals: convertRound(rustBracket.quarter_finals),
    semi_finals: convertRound(rustBracket.semi_finals),
    champion: convertSlot(rustBracket.champion ?? undefined),
    jointProbability: rustBracket.joint_probability,
    logProbability: rustBracket.log_probability,
  };

  return bracket;
}

/**
 * Verify that an optimal bracket has exactly 32 unique teams in R32.
 */
export function verifyOptimalBracket(bracket: OptimalBracketDisplay): { valid: boolean; errors: string[] } {
  const errors: string[] = [];
  const teamIds = new Set<number>();

  for (const match of bracket.round_of_32) {
    if (teamIds.has(match.teamA.teamId)) {
      errors.push(`Duplicate team in R32: ${match.teamA.team.code} (slot ${match.slot} team A)`);
    }
    teamIds.add(match.teamA.teamId);

    if (teamIds.has(match.teamB.teamId)) {
      errors.push(`Duplicate team in R32: ${match.teamB.team.code} (slot ${match.slot} team B)`);
    }
    teamIds.add(match.teamB.teamId);
  }

  if (teamIds.size !== 32) {
    errors.push(`Expected 32 unique teams in R32, found ${teamIds.size}`);
  }

  return { valid: errors.length === 0, errors };
}
