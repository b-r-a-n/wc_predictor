// Venue utility functions for resolving match participants

import type { Team, TeamProbability, AggregatedResults, BracketSlotStats, KnockoutRoundType } from '../types';

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
