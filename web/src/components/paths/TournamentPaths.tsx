import { useMemo, useCallback } from 'react';
import { useSimulatorStore } from '../../store/simulatorStore';
import { getFlagEmoji } from '../../utils/formatting';
import { PathTable } from './PathTable';
import type { Team, TopPathsResult, TournamentPathDisplay, KnockoutRoundType, Venue, PathEntry } from '../../types';

// Parse path string like "R32:5,R16:12,QF:3,SF:14,F:0" to extract round and opponent IDs
function parsePath(pathKey: string): { round: string; opponentId: number }[] {
  return pathKey.split(',').map(part => {
    const [round, id] = part.split(':');
    return { round, opponentId: parseInt(id) };
  });
}

// Map abbreviations to display names
const roundNames: Record<string, string> = {
  'R32': 'Round of 32',
  'R16': 'Round of 16',
  'QF': 'Quarter-finals',
  'SF': 'Semi-finals',
  'F': 'Final'
};

// Map abbreviations to KnockoutRoundType
const roundToType: Record<string, KnockoutRoundType> = {
  'R32': 'round_of_32',
  'R16': 'round_of_16',
  'QF': 'quarter_finals',
  'SF': 'semi_finals',
  'F': 'final'
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

  // Get top paths for the selected team
  // Extract directly from results.path_stats instead of calling WASM
  // (WASM can't deserialize back due to string key conversion)
  const pathsData = useMemo((): TopPathsResult | null => {
    if (!results || selectedTeamForPaths === null) return null;

    const pathStats = results.path_stats;
    if (!pathStats) {
      return null;
    }

    // path_stats keys are stringified TeamIds
    const teamPathStats = pathStats[String(selectedTeamForPaths)];
    if (!teamPathStats) {
      return {
        team_id: selectedTeamForPaths,
        total_simulations: results.total_simulations,
        has_paths: false,
        paths: []
      };
    }

    // Get complete_paths and sort by count descending
    const completePaths = teamPathStats.complete_paths;
    const sortedPaths = Object.entries(completePaths)
      .sort(([, a], [, b]) => b - a)
      .slice(0, 5);

    const paths: PathEntry[] = sortedPaths.map(([path, count]) => ({
      path,
      count,
      probability: count / results.total_simulations
    }));

    return {
      team_id: selectedTeamForPaths,
      total_simulations: results.total_simulations,
      has_paths: paths.length > 0,
      paths
    };
  }, [results, selectedTeamForPaths]);

  // Transform paths data into displayable format
  const displayPaths = useMemo((): TournamentPathDisplay[] => {
    if (!pathsData || !pathsData.has_paths) return [];

    return pathsData.paths.map((pathEntry, index) => {
      const parsedRounds = parsePath(pathEntry.path);

      const rounds = parsedRounds.map((pr, roundIndex) => {
        const roundType = roundToType[pr.round];
        const opponent = teamMap.get(pr.opponentId);

        // Try to get venue from mapping
        let venue: Venue | undefined;
        if (venueMapping && roundType) {
          // For simplicity, use match index 0 for each round since exact match position is not in path
          const roundMapping = venueMapping[roundType];
          if (roundMapping) {
            // Use roundIndex as approximate slot - this is a simplification
            const venueId = roundMapping[String(roundIndex % Object.keys(roundMapping).length)] || roundMapping['0'];
            if (venueId) {
              venue = venueMap.get(venueId);
            }
          }
        }

        return {
          round: roundType,
          roundDisplayName: roundNames[pr.round] || pr.round,
          opponentId: pr.opponentId,
          opponent,
          venue
        };
      });

      return {
        rank: index + 1,
        pathKey: pathEntry.path,
        probability: pathEntry.probability,
        occurrenceCount: pathEntry.count,
        rounds
      };
    });
  }, [pathsData, teamMap, venueMap, venueMapping]);

  const handleTeamChange = useCallback((e: React.ChangeEvent<HTMLSelectElement>) => {
    const value = e.target.value;
    setSelectedTeamForPaths(value === '' ? null : parseInt(value));
  }, [setSelectedTeamForPaths]);

  // Get selected team info for display
  const selectedTeam = selectedTeamForPaths !== null ? teamMap.get(selectedTeamForPaths) : null;

  // Show message if no simulation has been run
  if (!results) {
    return (
      <div className="flex items-center justify-center h-64 text-gray-500">
        Run a simulation to see tournament paths
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:justify-between sm:items-center gap-4">
        <div>
          <h2 className="text-lg font-semibold text-gray-900">Tournament Paths</h2>
          <p className="text-sm text-gray-500">
            View the most likely knockout stage paths for each team
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
                Top 5 most likely tournament paths
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
            <p>Select a team to view their tournament paths</p>
          </div>
        </div>
      )}

      {/* Error state - WASM call failed or no path_stats */}
      {selectedTeamForPaths !== null && !pathsData && (
        <div className="flex items-center justify-center h-48 text-amber-600 bg-amber-50 rounded-lg border border-amber-200">
          <div className="text-center">
            <span className="text-4xl block mb-2">&#9888;</span>
            <p className="font-medium">Path data not available</p>
            <p className="text-sm text-amber-500 mt-1">
              Please re-run the simulation to generate path statistics
            </p>
          </div>
        </div>
      )}

      {/* No paths available for selected team */}
      {selectedTeamForPaths !== null && pathsData && !pathsData.has_paths && (
        <div className="flex items-center justify-center h-48 text-gray-500 bg-gray-50 rounded-lg border border-gray-200">
          <div className="text-center">
            <span className="text-4xl block mb-2">&#128683;</span>
            <p>No knockout stage paths found for this team</p>
            <p className="text-sm text-gray-400 mt-1">
              This team may not have reached the knockout stage in any simulation
            </p>
          </div>
        </div>
      )}

      {/* Path table */}
      {displayPaths.length > 0 && (
        <PathTable
          paths={displayPaths}
          totalSimulations={pathsData?.total_simulations ?? 0}
        />
      )}
    </div>
  );
}
