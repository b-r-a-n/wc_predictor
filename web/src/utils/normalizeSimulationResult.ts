import type { AggregatedResults } from '../types';

type MaybeMap = Map<number, unknown> | Record<string, unknown> | undefined;

interface RawResult {
  total_simulations: number;
  team_stats: Map<number, unknown> | Record<string, unknown>;
  most_likely_winner: number;
  most_likely_final: [number, number];
  path_stats?: MaybeMap;
  bracket_slot_stats?: MaybeMap;
  bracket_slot_win_stats?: MaybeMap;
  slot_opponent_stats?: MaybeMap;
  optimal_bracket?: unknown;
}

function mapToObject(raw: MaybeMap): Record<string, unknown> | undefined {
  if (raw instanceof Map) {
    const out: Record<string, unknown> = {};
    raw.forEach((value: unknown, key: number) => {
      out[String(key)] = value;
    });
    return out;
  }
  if (raw) return raw as Record<string, unknown>;
  return undefined;
}

export function normalizeSimulationResult(rawResult: unknown): AggregatedResults {
  const result = rawResult as RawResult;

  const teamStats = mapToObject(result.team_stats) ?? {};
  const pathStats = mapToObject(result.path_stats);
  const bracketSlotStats = mapToObject(result.bracket_slot_stats);
  const bracketSlotWinStats = mapToObject(result.bracket_slot_win_stats);

  let slotOpponentStats: Record<string, unknown> | undefined;
  const rawSlotOpponentStats = result.slot_opponent_stats;
  if (rawSlotOpponentStats instanceof Map) {
    slotOpponentStats = {};
    rawSlotOpponentStats.forEach((teamStatsVal: unknown, teamId: number) => {
      slotOpponentStats![String(teamId)] = convertSlotOpponentStats(teamStatsVal);
    });
  } else if (rawSlotOpponentStats) {
    slotOpponentStats = rawSlotOpponentStats as Record<string, unknown>;
  }

  return {
    total_simulations: result.total_simulations,
    team_stats: teamStats,
    most_likely_winner: result.most_likely_winner,
    most_likely_final: result.most_likely_final,
    path_stats: pathStats,
    bracket_slot_stats: bracketSlotStats,
    bracket_slot_win_stats: bracketSlotWinStats,
    slot_opponent_stats: slotOpponentStats,
    optimal_bracket: result.optimal_bracket,
  } as AggregatedResults;
}

function convertSlotOpponentStats(teamStatsVal: unknown): Record<string, unknown> {
  const converted: Record<string, unknown> = {};
  const stats = teamStatsVal as Record<string, unknown>;

  for (const roundKey of ['round_of_32', 'round_of_16', 'quarter_finals', 'semi_finals']) {
    const roundData = stats[roundKey];
    if (roundData instanceof Map) {
      const roundConverted: Record<string, Record<string, number>> = {};
      roundData.forEach((opponentMap: unknown, slot: number) => {
        if (opponentMap instanceof Map) {
          const opponentConverted: Record<string, number> = {};
          (opponentMap as Map<number, number>).forEach((count, oppId) => {
            opponentConverted[String(oppId)] = count;
          });
          roundConverted[String(slot)] = opponentConverted;
        }
      });
      converted[roundKey] = roundConverted;
    } else if (roundData) {
      converted[roundKey] = roundData;
    }
  }

  const finalData = stats.final_match;
  if (finalData instanceof Map) {
    const finalConverted: Record<string, number> = {};
    (finalData as Map<number, number>).forEach((count, oppId) => {
      finalConverted[String(oppId)] = count;
    });
    converted.final_match = finalConverted;
  } else if (finalData) {
    converted.final_match = finalData;
  }

  return converted;
}
