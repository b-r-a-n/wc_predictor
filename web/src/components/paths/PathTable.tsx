import { getFlagEmoji, formatPercent, formatNumber } from '../../utils/formatting';
import type { TournamentPathDisplay } from '../../types';

interface PathTableProps {
  paths: TournamentPathDisplay[];
  totalSimulations: number;
}

export function PathTable({ paths, totalSimulations }: PathTableProps) {
  return (
    <div className="space-y-4">
      {/* Summary header */}
      <div className="text-sm text-gray-500">
        Based on {formatNumber(totalSimulations)} simulations
      </div>

      {/* Path cards */}
      {paths.map((path) => (
        <PathCard key={path.pathKey} path={path} />
      ))}
    </div>
  );
}

interface PathCardProps {
  path: TournamentPathDisplay;
}

function PathCard({ path }: PathCardProps) {
  // Determine medal/rank styling
  const getRankStyle = (rank: number) => {
    switch (rank) {
      case 1:
        return 'bg-yellow-100 text-yellow-800 border-yellow-300';
      case 2:
        return 'bg-gray-100 text-gray-700 border-gray-300';
      case 3:
        return 'bg-orange-100 text-orange-800 border-orange-300';
      default:
        return 'bg-blue-50 text-blue-700 border-blue-200';
    }
  };

  const getRankLabel = (rank: number) => {
    switch (rank) {
      case 1:
        return '#1';
      case 2:
        return '#2';
      case 3:
        return '#3';
      default:
        return `#${rank}`;
    }
  };

  return (
    <div className="bg-white rounded-lg border border-gray-200 shadow-sm overflow-hidden">
      {/* Card header */}
      <div className="flex items-center justify-between px-4 py-3 bg-gray-50 border-b border-gray-200">
        <div className="flex items-center gap-3">
          <span
            className={`inline-flex items-center justify-center w-8 h-8 rounded-full text-sm font-bold border ${getRankStyle(path.rank)}`}
          >
            {getRankLabel(path.rank)}
          </span>
          <div>
            <span className="text-lg font-semibold text-gray-900">
              {formatPercent(path.probability)}
            </span>
            <span className="text-gray-500 text-sm ml-2">
              probability
            </span>
          </div>
        </div>
        <div className="text-sm text-gray-500">
          {formatNumber(path.occurrenceCount)} occurrences
        </div>
      </div>

      {/* Round rows */}
      <div className="divide-y divide-gray-100">
        {path.rounds.map((round, index) => (
          <RoundRow
            key={`${path.pathKey}-${round.round}-${index}`}
            round={round}
            isLast={index === path.rounds.length - 1}
          />
        ))}
      </div>

      {/* Winner indicator for championship path */}
      {path.rounds.length === 5 && (
        <div className="px-4 py-3 bg-gradient-to-r from-yellow-50 to-yellow-100 border-t border-yellow-200">
          <div className="flex items-center gap-2 text-yellow-800">
            <span className="text-xl">&#127942;</span>
            <span className="font-semibold">Champion</span>
          </div>
        </div>
      )}
    </div>
  );
}

interface RoundRowProps {
  round: TournamentPathDisplay['rounds'][0];
  isLast: boolean;
}

function RoundRow({ round }: RoundRowProps) {
  return (
    <div className="flex items-center px-4 py-3 hover:bg-gray-50 transition-colors">
      {/* Round name */}
      <div className="w-32 flex-shrink-0">
        <span className="text-sm font-medium text-gray-700">
          {round.roundDisplayName}
        </span>
      </div>

      {/* Opponent */}
      <div className="flex-1 flex items-center gap-2 min-w-0">
        <span className="text-gray-400 text-sm">vs</span>
        {round.opponent ? (
          <>
            <span className="text-lg">{getFlagEmoji(round.opponent.code)}</span>
            <span className="text-sm font-medium text-gray-900 truncate">
              {round.opponent.name}
            </span>
          </>
        ) : (
          <span className="text-sm text-gray-400 italic">Unknown opponent</span>
        )}
      </div>

      {/* Venue */}
      <div className="hidden md:block flex-shrink-0 text-right max-w-xs">
        {round.venue ? (
          <div className="text-sm text-gray-500">
            <span className="font-medium">{round.venue.name}</span>
            <span className="text-gray-400">, {round.venue.city}</span>
          </div>
        ) : (
          <span className="text-sm text-gray-400 italic">--</span>
        )}
      </div>
    </div>
  );
}
