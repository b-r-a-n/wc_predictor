import { useMemo, useCallback, useState } from 'react';
import { useSimulatorStore } from '../../store/simulatorStore';
import { getFlagEmoji, formatNumber } from '../../utils/formatting';
import { BracketSlot } from './BracketSlot';
import { BracketConnectors } from './BracketConnectors';
import {
  calculateSlotPositions,
  calculateBracketDimensions,
} from './bracketLayout';
import type { Team, Venue, BracketSlotStats, KnockoutRoundType, PathStatistics, SlotOpponentStats } from '../../types';

// Round configuration for the bracket
const ROUNDS: {
  key: keyof BracketSlotStats;
  displayName: string;
  slotCount: number;
  mappingKey: KnockoutRoundType;
  roundIndex: number;
}[] = [
  { key: 'round_of_32', displayName: 'R32', slotCount: 16, mappingKey: 'round_of_32', roundIndex: 0 },
  { key: 'round_of_16', displayName: 'R16', slotCount: 8, mappingKey: 'round_of_16', roundIndex: 1 },
  { key: 'quarter_finals', displayName: 'QF', slotCount: 4, mappingKey: 'quarter_finals', roundIndex: 2 },
  { key: 'semi_finals', displayName: 'SF', slotCount: 2, mappingKey: 'semi_finals', roundIndex: 3 },
  { key: 'final_match', displayName: 'Final', slotCount: 1, mappingKey: 'final', roundIndex: 4 },
];

// Map PathStatistics round keys to BracketSlotStats round keys
const PATH_ROUND_TO_BRACKET_ROUND: Record<string, keyof BracketSlotStats> = {
  round_of_32_matchups: 'round_of_32',
  round_of_16_matchups: 'round_of_16',
  quarter_final_matchups: 'quarter_finals',
  semi_final_matchups: 'semi_finals',
  final_matchups: 'final_match',
};

export function TournamentPaths() {
  const {
    results,
    teams,
    venues,
    venueMapping,
    selectedTeamForPaths,
    setSelectedTeamForPaths
  } = useSimulatorStore();

  // Track which slot is expanded
  const [expandedSlot, setExpandedSlot] = useState<string | null>(null);

  // Create team lookup map
  const teamMap = useMemo(() => {
    const map = new Map<number, Team>();
    teams.forEach((t) => map.set(t.id, t));
    return map;
  }, [teams]);

  // Create venue lookup map
  const venueMap = useMemo(() => {
    if (!venues) return new Map<string, Venue>();
    const map = new Map<string, Venue>();
    venues.forEach((v) => map.set(v.id, v));
    return map;
  }, [venues]);

  // Calculate bracket layout positions
  const slotPositions = useMemo(() => calculateSlotPositions(), []);
  const bracketDimensions = useMemo(() => calculateBracketDimensions(slotPositions), [slotPositions]);

  // Get bracket slot stats for the selected team
  const bracketStats = useMemo((): BracketSlotStats | null => {
    if (!results || selectedTeamForPaths === null) return null;

    const bracketSlotStats = results.bracket_slot_stats;
    if (!bracketSlotStats) return null;

    const teamStats = bracketSlotStats[String(selectedTeamForPaths)] as BracketSlotStats | undefined;
    return teamStats || null;
  }, [results, selectedTeamForPaths]);

  // Get path stats for the selected team (for opponent info - legacy, round-level)
  const pathStats = useMemo((): PathStatistics | null => {
    if (!results || selectedTeamForPaths === null) return null;

    const allPathStats = results.path_stats;
    if (!allPathStats) return null;

    const teamPathStats = allPathStats[String(selectedTeamForPaths)] as PathStatistics | undefined;
    return teamPathStats || null;
  }, [results, selectedTeamForPaths]);

  // Get slot-specific opponent stats for the selected team
  const slotOpponentStats = useMemo((): SlotOpponentStats | null => {
    if (!results || selectedTeamForPaths === null) return null;

    const allSlotOpponentStats = results.slot_opponent_stats;
    if (!allSlotOpponentStats) return null;

    const teamStats = allSlotOpponentStats[String(selectedTeamForPaths)] as SlotOpponentStats | undefined;
    return teamStats || null;
  }, [results, selectedTeamForPaths]);

  // Get highlighted slots (probability > 1%)
  const highlightedSlots = useMemo(() => {
    const highlighted = new Set<string>();
    if (!bracketStats || !results) return highlighted;

    for (const round of ROUNDS) {
      if (round.key === 'final_match') {
        const count = bracketStats.final_match as number;
        const prob = count / results.total_simulations;
        if (prob >= 0.01) {
          highlighted.add(`${round.roundIndex}-0`);
        }
      } else {
        const roundData = bracketStats[round.key] as Record<string, number>;
        if (roundData) {
          for (const [slotStr, count] of Object.entries(roundData)) {
            const prob = count / results.total_simulations;
            if (prob >= 0.01) {
              highlighted.add(`${round.roundIndex}-${slotStr}`);
            }
          }
        }
      }
    }

    return highlighted;
  }, [bracketStats, results]);

  // Get probability for a specific slot
  const getSlotProbability = useCallback((
    roundKey: keyof BracketSlotStats,
    slotIndex: number
  ): number => {
    if (!bracketStats || !results) return 0;

    if (roundKey === 'final_match') {
      const count = bracketStats.final_match as number;
      return count / results.total_simulations;
    }

    const roundData = bracketStats[roundKey] as Record<string, number>;
    if (!roundData) return 0;

    const count = roundData[String(slotIndex)] || 0;
    return count / results.total_simulations;
  }, [bracketStats, results]);

  // Get venue for a specific slot
  const getSlotVenue = useCallback((
    mappingKey: KnockoutRoundType,
    slotIndex: number
  ): Venue | undefined => {
    if (!venueMapping) return undefined;

    const roundMapping = venueMapping[mappingKey];
    if (!roundMapping) return undefined;

    const venueId = roundMapping[String(slotIndex)];
    if (!venueId) return undefined;

    return venueMap.get(venueId);
  }, [venueMapping, venueMap]);

  // Get top opponents for a specific slot based on slot-specific stats
  // Falls back to round-level stats if slot-specific data is unavailable
  const getTopOpponents = useCallback((
    roundKey: keyof BracketSlotStats,
    slotIndex: number
  ): { team: Team; probability: number }[] => {
    if (!results) return [];

    // Try slot-specific data first (new format)
    if (slotOpponentStats) {
      let opponentCounts: Record<string, number> | undefined;

      if (roundKey === 'final_match') {
        // Final has a single slot, use final_match directly
        opponentCounts = slotOpponentStats.final_match;
      } else {
        // Get the slot-specific opponents for other rounds
        const roundData = slotOpponentStats[roundKey] as Record<string, Record<string, number>> | undefined;
        if (roundData) {
          opponentCounts = roundData[String(slotIndex)];
        }
      }

      if (opponentCounts && Object.keys(opponentCounts).length > 0) {
        const sorted = Object.entries(opponentCounts)
          .map(([teamIdStr, count]) => ({
            teamId: parseInt(teamIdStr),
            count: count as number,
          }))
          .sort((a, b) => b.count - a.count)
          .slice(0, 5);

        return sorted
          .map(({ teamId, count }) => {
            const team = teamMap.get(teamId);
            if (!team) return null;
            return {
              team,
              probability: count / results.total_simulations,
            };
          })
          .filter((item): item is { team: Team; probability: number } => item !== null);
      }
    }

    // Fallback to round-level path stats (legacy behavior)
    if (!pathStats) return [];

    // Map bracket round key to path stats key
    const pathRoundKey = Object.entries(PATH_ROUND_TO_BRACKET_ROUND)
      .find(([, v]) => v === roundKey)?.[0] as keyof PathStatistics | undefined;

    if (!pathRoundKey) return [];

    const roundMatchups = pathStats[pathRoundKey];
    if (!roundMatchups || typeof roundMatchups !== 'object' || !('opponents' in roundMatchups)) {
      return [];
    }

    const opponents = (roundMatchups as { opponents: Record<string, number> }).opponents;
    if (!opponents) return [];

    // Sort by count and get top opponents
    const sorted = Object.entries(opponents)
      .map(([teamIdStr, count]) => ({
        teamId: parseInt(teamIdStr),
        count: count as number,
      }))
      .sort((a, b) => b.count - a.count)
      .slice(0, 5);

    return sorted
      .map(({ teamId, count }) => {
        const team = teamMap.get(teamId);
        if (!team) return null;
        return {
          team,
          probability: count / results.total_simulations,
        };
      })
      .filter((item): item is { team: Team; probability: number } => item !== null);
  }, [slotOpponentStats, pathStats, results, teamMap]);

  const handleTeamChange = useCallback((e: React.ChangeEvent<HTMLSelectElement>) => {
    const value = e.target.value;
    setSelectedTeamForPaths(value === '' ? null : parseInt(value));
    setExpandedSlot(null);  // Reset expanded slot when team changes
  }, [setSelectedTeamForPaths]);

  const handleToggleExpand = useCallback((slotKey: string) => {
    setExpandedSlot((prev) => (prev === slotKey ? null : slotKey));
  }, []);

  // Get selected team info for display
  const selectedTeam = selectedTeamForPaths !== null ? teamMap.get(selectedTeamForPaths) : null;

  // Check if team has any bracket slot data
  const hasData = bracketStats !== null;

  // Show message if no simulation has been run
  if (!results) {
    return (
      <div className="flex items-center justify-center h-64 text-gray-500">
        Run a simulation to see bracket positions
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:justify-between sm:items-center gap-4">
        <div>
          <h2 className="text-lg font-semibold text-gray-900">Tournament Bracket</h2>
          <p className="text-sm text-gray-500">
            View bracket positions and opponents for a team
          </p>
        </div>

        {/* Team Selector */}
        <div className="flex items-center gap-2">
          <label htmlFor="team-select" className="text-sm font-medium text-gray-700">
            Select Team:
          </label>
          <select
            id="team-select"
            value={selectedTeamForPaths ?? ''}
            onChange={handleTeamChange}
            className="block w-48 rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 text-sm"
          >
            <option value="">-- Choose a team --</option>
            {teams
              .slice()
              .sort((a, b) => a.name.localeCompare(b.name))
              .map((team) => (
                <option key={team.id} value={team.id}>
                  {getFlagEmoji(team.code)} {team.name}
                </option>
              ))}
          </select>
        </div>
      </div>

      {/* Selected team header */}
      {selectedTeam && (
        <div className="bg-gradient-to-r from-blue-500 to-blue-600 rounded-lg p-4 text-white">
          <div className="flex items-center gap-3">
            <span className="text-3xl">{getFlagEmoji(selectedTeam.code)}</span>
            <div>
              <h3 className="text-xl font-bold">{selectedTeam.name}</h3>
              <p className="text-blue-100 text-sm">
                Bracket position probabilities - hover for match info, click for opponents
              </p>
            </div>
          </div>
        </div>
      )}

      {/* No team selected state */}
      {selectedTeamForPaths === null && (
        <div className="flex items-center justify-center h-48 text-gray-500 bg-gray-50 rounded-lg border border-gray-200">
          <div className="text-center">
            <span className="text-4xl block mb-2">&#127942;</span>
            <p>Select a team to view their bracket positions</p>
          </div>
        </div>
      )}

      {/* No data available for selected team */}
      {selectedTeamForPaths !== null && !hasData && (
        <div className="flex items-center justify-center h-48 text-amber-600 bg-amber-50 rounded-lg border border-amber-200">
          <div className="text-center">
            <span className="text-4xl block mb-2">&#9888;</span>
            <p className="font-medium">Bracket data not available</p>
            <p className="text-sm text-amber-500 mt-1">
              This team may not have reached the knockout stage in any simulation
            </p>
          </div>
        </div>
      )}

      {/* Bracket visualization */}
      {selectedTeamForPaths !== null && hasData && (
        <>
          {/* Summary header */}
          <div className="text-sm text-gray-500">
            Based on {formatNumber(results.total_simulations)} simulations
          </div>

          {/* Bracket tree visualization */}
          <div className="bg-white rounded-lg border border-gray-200 p-4 overflow-x-auto">
            <div
              className="relative"
              style={{
                width: bracketDimensions.width,
                height: bracketDimensions.height,
                minWidth: '900px',
              }}
            >
              {/* Round headers */}
              {ROUNDS.map((round, idx) => {
                const roundPositions = slotPositions.filter(p => p.round === idx);
                if (roundPositions.length === 0) return null;
                const firstPos = roundPositions[0];

                return (
                  <div
                    key={round.key}
                    className="absolute text-center font-semibold text-gray-700 text-sm py-2 bg-gray-100 rounded-md"
                    style={{
                      left: firstPos.x,
                      top: 0,
                      width: firstPos.width,
                    }}
                  >
                    {round.displayName}
                  </div>
                );
              })}

              {/* SVG Connectors */}
              <BracketConnectors
                positions={slotPositions}
                width={bracketDimensions.width}
                height={bracketDimensions.height}
                highlightedSlots={highlightedSlots}
              />

              {/* Bracket slots */}
              {slotPositions.map((pos) => {
                const round = ROUNDS[pos.round];
                if (!round) return null;

                const slotKey = `${round.key}-${pos.slot}`;
                const probability = getSlotProbability(round.key, pos.slot);
                const venue = getSlotVenue(round.mappingKey, pos.slot);
                const topOpponents = getTopOpponents(round.key, pos.slot);
                const isExpanded = expandedSlot === slotKey;

                return (
                  <div
                    key={slotKey}
                    className="absolute"
                    style={{
                      left: pos.x,
                      top: pos.y,
                      width: pos.width,
                      zIndex: isExpanded ? 20 : 10,
                    }}
                  >
                    <BracketSlot
                      round={round.key}
                      slotIndex={pos.slot}
                      probability={probability}
                      totalSimulations={results.total_simulations}
                      venue={venue}
                      topOpponents={topOpponents}
                      isExpanded={isExpanded}
                      onToggleExpand={() => handleToggleExpand(slotKey)}
                    />
                  </div>
                );
              })}
            </div>
          </div>

          {/* Legend */}
          <div className="bg-gray-50 rounded-lg border border-gray-200 p-4">
            <h4 className="text-sm font-semibold text-gray-700 mb-3">Probability Legend</h4>
            <div className="flex flex-wrap gap-4">
              <div className="flex items-center gap-2">
                <div className="w-6 h-6 rounded bg-green-500"></div>
                <span className="text-sm text-gray-600">&gt;20%</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="w-6 h-6 rounded bg-green-300"></div>
                <span className="text-sm text-gray-600">10-20%</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="w-6 h-6 rounded bg-yellow-300"></div>
                <span className="text-sm text-gray-600">5-10%</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="w-6 h-6 rounded bg-gray-200"></div>
                <span className="text-sm text-gray-600">1-5%</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="w-6 h-6 rounded bg-gray-100 border border-gray-200"></div>
                <span className="text-sm text-gray-600">&lt;1%</span>
              </div>
            </div>
            <p className="text-xs text-gray-500 mt-3">
              Hover over R32 slots to see which group positions feed into each match. Click any slot to view top opponents.
            </p>
          </div>
        </>
      )}
    </div>
  );
}
