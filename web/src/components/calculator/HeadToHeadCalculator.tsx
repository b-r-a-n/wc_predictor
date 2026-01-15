import { useState, useMemo } from 'react';
import { useSimulatorStore } from '../../store/simulatorStore';
import { TeamSelector } from './TeamSelector';
import { getFlagEmoji, formatPercent } from '../../utils/formatting';
import type { Team, MatchProbabilities } from '../../types';

export function HeadToHeadCalculator() {
  const { teams, wasmApi } = useSimulatorStore();
  const [teamA, setTeamA] = useState<Team | null>(null);
  const [teamB, setTeamB] = useState<Team | null>(null);
  const [isKnockout, setIsKnockout] = useState(false);

  const probabilities: MatchProbabilities | null = useMemo(() => {
    if (!teamA || !teamB || !wasmApi) return null;
    return wasmApi.calculateMatchProbability(teamA.elo_rating, teamB.elo_rating, isKnockout);
  }, [teamA, teamB, isKnockout, wasmApi]);

  return (
    <div className="max-w-2xl mx-auto space-y-6">
      <div>
        <h2 className="text-lg font-semibold text-gray-900">Head-to-Head Calculator</h2>
        <p className="text-sm text-gray-500 mt-1">
          Calculate match probabilities between any two teams based on ELO ratings
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <TeamSelector
          teams={teams}
          selectedTeam={teamA}
          onChange={setTeamA}
          label="Team A (Home)"
          excludeTeamId={teamB?.id}
        />
        <TeamSelector
          teams={teams}
          selectedTeam={teamB}
          onChange={setTeamB}
          label="Team B (Away)"
          excludeTeamId={teamA?.id}
        />
      </div>

      {/* Match type toggle */}
      <div className="flex items-center gap-4">
        <span className="text-sm font-medium text-gray-700">Match Type:</span>
        <div className="flex rounded-lg overflow-hidden border border-gray-200">
          <button
            className={`px-4 py-2 text-sm font-medium ${
              !isKnockout
                ? 'bg-blue-600 text-white'
                : 'bg-white text-gray-700 hover:bg-gray-50'
            }`}
            onClick={() => setIsKnockout(false)}
          >
            Group Stage
          </button>
          <button
            className={`px-4 py-2 text-sm font-medium ${
              isKnockout
                ? 'bg-blue-600 text-white'
                : 'bg-white text-gray-700 hover:bg-gray-50'
            }`}
            onClick={() => setIsKnockout(true)}
          >
            Knockout
          </button>
        </div>
      </div>

      {/* Results */}
      {probabilities && teamA && teamB && (
        <div className="bg-white rounded-lg shadow-md p-6 space-y-4">
          {/* Team comparison header */}
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span className="text-3xl">{getFlagEmoji(teamA.code)}</span>
              <div>
                <div className="font-bold text-gray-900">{teamA.name}</div>
                <div className="text-sm text-gray-500">ELO: {teamA.elo_rating}</div>
              </div>
            </div>
            <div className="text-2xl font-bold text-gray-400">vs</div>
            <div className="flex items-center gap-2">
              <div className="text-right">
                <div className="font-bold text-gray-900">{teamB.name}</div>
                <div className="text-sm text-gray-500">ELO: {teamB.elo_rating}</div>
              </div>
              <span className="text-3xl">{getFlagEmoji(teamB.code)}</span>
            </div>
          </div>

          {/* Probability bar */}
          <div className="relative h-12 rounded-lg overflow-hidden flex">
            <div
              className="bg-green-500 flex items-center justify-center text-white font-bold"
              style={{ width: `${probabilities.home_win * 100}%` }}
            >
              {probabilities.home_win > 0.1 && formatPercent(probabilities.home_win)}
            </div>
            {!isKnockout && (
              <div
                className="bg-gray-400 flex items-center justify-center text-white font-bold"
                style={{ width: `${probabilities.draw * 100}%` }}
              >
                {probabilities.draw > 0.1 && formatPercent(probabilities.draw)}
              </div>
            )}
            <div
              className="bg-blue-500 flex items-center justify-center text-white font-bold"
              style={{ width: `${(isKnockout ? 1 - probabilities.home_win : probabilities.away_win) * 100}%` }}
            >
              {(isKnockout ? 1 - probabilities.home_win : probabilities.away_win) > 0.1 &&
                formatPercent(isKnockout ? 1 - probabilities.home_win : probabilities.away_win)}
            </div>
          </div>

          {/* Legend */}
          <div className="flex justify-between text-sm">
            <div className="flex items-center gap-2">
              <span className="w-4 h-4 rounded bg-green-500" />
              <span>
                {teamA.name} wins: {formatPercent(probabilities.home_win)}
              </span>
            </div>
            {!isKnockout && (
              <div className="flex items-center gap-2">
                <span className="w-4 h-4 rounded bg-gray-400" />
                <span>Draw: {formatPercent(probabilities.draw)}</span>
              </div>
            )}
            <div className="flex items-center gap-2">
              <span className="w-4 h-4 rounded bg-blue-500" />
              <span>
                {teamB.name} wins:{' '}
                {formatPercent(isKnockout ? 1 - probabilities.home_win : probabilities.away_win)}
              </span>
            </div>
          </div>

          {/* ELO difference explanation */}
          <div className="text-sm text-gray-500 bg-gray-50 rounded p-3">
            <strong>ELO Difference:</strong> {Math.abs(teamA.elo_rating - teamB.elo_rating)} points
            {teamA.elo_rating > teamB.elo_rating
              ? ` in favor of ${teamA.name}`
              : teamA.elo_rating < teamB.elo_rating
                ? ` in favor of ${teamB.name}`
                : ' (teams are evenly matched)'}
            {isKnockout && (
              <>
                <br />
                <em>Knockout matches have no draws - extra time and penalties decide the winner.</em>
              </>
            )}
          </div>
        </div>
      )}

      {!teamA || !teamB ? (
        <div className="text-center py-8 text-gray-500">
          Select two teams to see match probabilities
        </div>
      ) : null}
    </div>
  );
}
