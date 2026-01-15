import type { Team } from '../../types';
import { getFlagEmoji, formatPercent } from '../../utils/formatting';

interface TeamSlotProps {
  team: Team | null;
  probability?: number;
  isWinner?: boolean;
  size?: 'sm' | 'md';
}

export function TeamSlot({ team, probability, isWinner = false, size = 'md' }: TeamSlotProps) {
  const sizeClasses = {
    sm: 'px-2 py-1 text-xs',
    md: 'px-3 py-2 text-sm',
  };

  if (!team) {
    return (
      <div
        className={`${sizeClasses[size]} bg-gray-100 rounded border border-gray-200 text-gray-400`}
      >
        TBD
      </div>
    );
  }

  return (
    <div
      className={`${sizeClasses[size]} rounded border ${
        isWinner ? 'bg-green-50 border-green-500' : 'bg-white border-gray-200'
      } flex items-center justify-between gap-2`}
    >
      <div className="flex items-center gap-1.5">
        <span className={size === 'sm' ? 'text-sm' : 'text-base'}>{getFlagEmoji(team.code)}</span>
        <span className={`font-medium ${isWinner ? 'text-green-800' : 'text-gray-900'}`}>
          {team.code}
        </span>
      </div>
      {probability !== undefined && (
        <span className="text-gray-500 font-medium">{formatPercent(probability, 0)}</span>
      )}
    </div>
  );
}
