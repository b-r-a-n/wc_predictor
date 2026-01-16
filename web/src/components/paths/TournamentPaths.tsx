import { useMemo, useCallback } from 'react';
import { useSimulatorStore } from '../../store/simulatorStore';
import { getFlagEmoji, formatNumber } from '../../utils/formatting';
import { BracketSlot } from './BracketSlot';
import type { Team, Venue, BracketSlotStats, KnockoutRoundType } from '../../types';

// Round configuration for the bracket
const ROUNDS: {
  key: keyof BracketSlotStats;
  displayName: string;
  slotCount: number;
  mappingKey: KnockoutRoundType;
}[] = [
  { key: 'round_of_32', displayName: 'R32', slotCount: 16, mappingKey: 'round_of_32' },
  { key: 'round_of_16', displayName: 'R16', slotCount: 8, mappingKey: 'round_of_16' },
  { key: 'quarter_finals', displayName: 'QF', slotCount: 4, mappingKey: 'quarter_finals' },
  { key: 'semi_finals', displayName: 'SF', slotCount: 2, mappingKey: 'semi_finals' },
  { key: 'final_match', displayName: 'Final', slotCount: 1, mappingKey: 'final' },
];

export function TournamentPaths() {
  const {
    results,
    teams,
    venues,
    venueMapping,
    selectedTeamForPaths,
    setSelectedTeamForPaths
  } = useSimulatorStore();

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

  // Get bracket slot stats for the selected team
  const bracketStats = useMemo((): BracketSlotStats | null => {
    if (!results || selectedTeamForPaths === null) return null;

    const bracketSlotStats = results.bracket_slot_stats;
    if (!bracketSlotStats) return null;

    const teamStats = bracketSlotStats[String(selectedTeamForPaths)] as BracketSlotStats | undefined;
    return teamStats || null;
  }, [results, selectedTeamForPaths]);

  // Get probability for a specific slot
  const getSlotProbability = useCallback((
    roundKey: keyof BracketSlotStats,
    slotIndex: number
  ): number => {
    if (!bracketStats || !results) return 0;

    if (roundKey === 'final_match') {
      // Final is a single number, not a record
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

  const handleTeamChange = useCallback((e: React.ChangeEvent<HTMLSelectElement>) => {
    const value = e.target.value;
    setSelectedTeamForPaths(value === '' ? null : parseInt(value));
  }, [setSelectedTeamForPaths]);

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
          <h2 className="text-lg font-semibold text-gray-900">Bracket Positions</h2>
          <p className="text-sm text-gray-500">
            View probability of each bracket position for a team
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
                Bracket position probabilities
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

          {/* Bracket grid */}
          <div className="bg-white rounded-lg border border-gray-200 p-4 overflow-x-auto">
            <div className="flex gap-4 min-w-max">
              {ROUNDS.map((round) => (
                <div key={round.key} className="flex flex-col gap-2" style={{ width: '140px' }}>
                  {/* Round header */}
                  <div className="text-center font-semibold text-gray-700 text-sm py-2 bg-gray-100 rounded-md">
                    {round.displayName}
                  </div>

                  {/* Slots for this round */}
                  <div className="flex flex-col gap-2">
                    {Array.from({ length: round.slotCount }, (_, slotIndex) => (
                      <BracketSlot
                        key={`${round.key}-${slotIndex}`}
                        round={round.key}
                        slotIndex={slotIndex}
                        probability={getSlotProbability(round.key, slotIndex)}
                        totalSimulations={results.total_simulations}
                        venue={getSlotVenue(round.mappingKey, slotIndex)}
                      />
                    ))}
                  </div>
                </div>
              ))}
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
          </div>
        </>
      )}
    </div>
  );
}
