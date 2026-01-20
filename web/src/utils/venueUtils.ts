// Venue utility functions for resolving match participants

import type { Team, TeamProbability, AggregatedResults, BracketSlotStats, KnockoutRoundType, SlotOpponentStats } from '../types';

/**
 * Represents a specific knockout pairing (two teams that face each other)
 */
export interface KnockoutPairing {
  teamA: Team;
  teamB: Team;
  probability: number;
  count: number;
}

/**
 * Threshold for displaying a team as a candidate (5%)
 */
const PROBABILITY_THRESHOLD = 0.05;

/**
 * Maps a knockout round type to the key in BracketSlotStats
 */
function getRoundKey(round: KnockoutRoundType): keyof BracketSlotStats {
  switch (round) {
    case 'round_of_32':
      return 'round_of_32';
    case 'round_of_16':
      return 'round_of_16';
    case 'quarter_finals':
      return 'quarter_finals';
    case 'semi_finals':
      return 'semi_finals';
    case 'third_place':
    case 'final':
      return 'final_match';
    default:
      return 'round_of_32';
  }
}

/**
 * Resolves teams that could potentially appear in a knockout slot based on simulation results.
 * Returns teams with probability >= threshold, sorted by probability descending.
 *
 * @param round - The knockout round type
 * @param slot - The slot index within the round
 * @param results - Aggregated simulation results
 * @param teams - Array of all teams
 * @param threshold - Minimum probability to include (default 5%)
 * @returns Array of teams with their probabilities, sorted by probability descending
 */
export function resolveKnockoutMatchCandidates(
  round: KnockoutRoundType,
  slot: number,
  results: AggregatedResults | null,
  teams: Team[],
  threshold: number = PROBABILITY_THRESHOLD
): TeamProbability[] {
  if (!results || !results.bracket_slot_stats) {
    return [];
  }

  const totalSimulations = results.total_simulations;
  if (totalSimulations === 0) {
    return [];
  }

  const roundKey = getRoundKey(round);
  const candidates: TeamProbability[] = [];

  // Iterate through all teams' bracket slot stats
  for (const [teamIdStr, slotStats] of Object.entries(results.bracket_slot_stats)) {
    const teamId = parseInt(teamIdStr, 10);
    const team = teams.find(t => t.id === teamId);
    if (!team) continue;

    const stats = slotStats as BracketSlotStats;

    // For final and third place, we use final_match which is a count not a record
    if (roundKey === 'final_match') {
      // For the final, check if the team appeared in the final
      const count = stats.final_match || 0;
      const probability = count / totalSimulations;
      if (probability >= threshold) {
        candidates.push({ team, probability });
      }
    } else {
      // For other rounds, check the specific slot
      const roundStats = stats[roundKey] as Record<string, number>;
      if (roundStats) {
        const slotKey = String(slot);
        const count = roundStats[slotKey] || 0;
        const probability = count / totalSimulations;
        if (probability >= threshold) {
          candidates.push({ team, probability });
        }
      }
    }
  }

  // Sort by probability descending
  candidates.sort((a, b) => b.probability - a.probability);

  return candidates;
}

/**
 * Gets display name for a knockout round
 */
export function getRoundDisplayName(round: string): string {
  switch (round) {
    case 'group_stage':
      return 'Group Stage';
    case 'round_of_32':
      return 'Round of 32';
    case 'round_of_16':
      return 'Round of 16';
    case 'quarter_finals':
      return 'Quarter-Finals';
    case 'semi_finals':
      return 'Semi-Finals';
    case 'third_place':
      return 'Third Place Play-off';
    case 'final':
      return 'Final';
    default:
      return round;
  }
}

/**
 * Format a date string for display
 */
export function formatMatchDate(dateStr: string): string {
  const date = new Date(dateStr + 'T00:00:00');
  return date.toLocaleDateString('en-US', {
    weekday: 'short',
    month: 'short',
    day: 'numeric',
  });
}

/**
 * Format a time string for display (assumes 24h input like "13:00")
 */
export function formatMatchTime(timeStr: string): string {
  const [hours, minutes] = timeStr.split(':').map(Number);
  const period = hours >= 12 ? 'PM' : 'AM';
  const displayHours = hours % 12 || 12;
  return `${displayHours}:${minutes.toString().padStart(2, '0')} ${period}`;
}

/**
 * Resolves the most likely pairings (actual matchups) for a knockout match slot.
 * Uses slot_opponent_stats to find which teams faced each other in a specific slot.
 *
 * @param round - The knockout round type
 * @param slot - The slot index within the round
 * @param results - Aggregated simulation results
 * @param teams - Array of all teams
 * @param maxPairings - Maximum number of pairings to return (default 5)
 * @returns Array of pairings sorted by probability descending
 */
export function resolveKnockoutMatchPairings(
  round: KnockoutRoundType,
  slot: number,
  results: AggregatedResults | null,
  teams: Team[],
  maxPairings: number = 5
): KnockoutPairing[] {
  if (!results || !results.slot_opponent_stats) {
    return [];
  }

  const totalSimulations = results.total_simulations;
  if (totalSimulations === 0) {
    return [];
  }

  // Create team lookup
  const teamMap = new Map<number, Team>();
  for (const team of teams) {
    teamMap.set(team.id, team);
  }

  // Aggregate pairings using canonical key (smaller ID first) to avoid double-counting
  const pairingCounts = new Map<string, { teamAId: number; teamBId: number; count: number }>();

  const roundKey = getRoundKey(round);

  // Iterate through all teams' slot opponent stats
  for (const [teamIdStr, stats] of Object.entries(results.slot_opponent_stats)) {
    const teamId = parseInt(teamIdStr, 10);
    const slotStats = stats as SlotOpponentStats;

    // For final, use final_match directly (single slot)
    if (roundKey === 'final_match') {
      const opponents = slotStats.final_match;
      if (opponents) {
        for (const [oppIdStr, count] of Object.entries(opponents)) {
          const oppId = parseInt(oppIdStr, 10);
          // Use canonical key (smaller ID first)
          const [idA, idB] = teamId < oppId ? [teamId, oppId] : [oppId, teamId];
          const key = `${idA}-${idB}`;

          if (!pairingCounts.has(key)) {
            pairingCounts.set(key, { teamAId: idA, teamBId: idB, count: 0 });
          }
          // Each pairing is recorded twice (once by each team), so we only add half
          // But since we're iterating through all teams, we'll just track the max
          const existing = pairingCounts.get(key)!;
          // Take the count from one side (they should be equal)
          existing.count = Math.max(existing.count, count);
        }
      }
    } else {
      // For other rounds, get slot-specific opponent data
      const roundData = slotStats[roundKey as keyof Omit<SlotOpponentStats, 'final_match'>] as Record<string, Record<string, number>> | undefined;
      if (!roundData) continue;

      const slotData = roundData[String(slot)];
      if (!slotData) continue;

      for (const [oppIdStr, count] of Object.entries(slotData)) {
        const oppId = parseInt(oppIdStr, 10);
        // Use canonical key (smaller ID first)
        const [idA, idB] = teamId < oppId ? [teamId, oppId] : [oppId, teamId];
        const key = `${idA}-${idB}`;

        if (!pairingCounts.has(key)) {
          pairingCounts.set(key, { teamAId: idA, teamBId: idB, count: 0 });
        }
        const existing = pairingCounts.get(key)!;
        existing.count = Math.max(existing.count, count);
      }
    }
  }

  // Convert to array and sort by count
  const pairingsArray = Array.from(pairingCounts.values())
    .sort((a, b) => b.count - a.count)
    .slice(0, maxPairings);

  // Map to KnockoutPairing objects
  return pairingsArray
    .map(({ teamAId, teamBId, count }) => {
      const teamA = teamMap.get(teamAId);
      const teamB = teamMap.get(teamBId);
      if (!teamA || !teamB) return null;

      return {
        teamA,
        teamB,
        count,
        probability: count / totalSimulations,
      };
    })
    .filter((p): p is KnockoutPairing => p !== null);
}
