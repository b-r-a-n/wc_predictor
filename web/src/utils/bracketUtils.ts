import type { Team, AggregatedResults, MostLikelyBracket, MostLikelySlotData, RustMostLikelyBracketSlot } from '../types';

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
 * Convert the pre-computed Rust bracket to the UI format.
 * The greedy algorithm is now in Rust (aggregator.rs) which ensures:
 * 1. Each team appears at most once (no duplicates)
 * 2. Tournament structure is valid (later round winners must have won feeder matches)
 * 3. Higher-ranked teams get priority for their best slots
 * 4. Teams with no wins at a slot fall back to participation stats (fixes missing teams)
 */
export function computeMostLikelyBracket(
  results: AggregatedResults,
  teamMap: Map<number, Team>
): MostLikelyBracket | null {
  const rustBracket = results.most_likely_bracket;
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

  const bracket: MostLikelyBracket = {
    round_of_32: convertRound(rustBracket.round_of_32),
    round_of_16: convertRound(rustBracket.round_of_16),
    quarter_finals: convertRound(rustBracket.quarter_finals),
    semi_finals: convertRound(rustBracket.semi_finals),
    final_match: convertSlot(rustBracket.final_match ?? undefined),
    champion: convertSlot(rustBracket.champion ?? undefined),
  };

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
