import { formatPercent } from '../../utils/formatting';
import type { Venue } from '../../types';

interface BracketSlotProps {
  round: string;           // "round_of_32", "round_of_16", etc.
  slotIndex: number;       // 0-15 for R32, 0-7 for R16, etc.
  probability: number;     // 0.0 to 1.0
  totalSimulations: number;
  venue?: Venue;
}

// Get background color class based on probability
function getProbabilityColorClass(probability: number): string {
  if (probability > 0.20) {
    return 'bg-green-500 text-white';
  } else if (probability >= 0.10) {
    return 'bg-green-300 text-gray-900';
  } else if (probability >= 0.05) {
    return 'bg-yellow-300 text-gray-900';
  } else if (probability >= 0.01) {
    return 'bg-gray-200 text-gray-700';
  } else {
    return 'bg-gray-100 text-gray-400';
  }
}

export function BracketSlot({
  probability,
  venue,
}: BracketSlotProps) {
  const colorClass = getProbabilityColorClass(probability);
  const isMuted = probability < 0.01;

  return (
    <div
      className={`rounded-md p-2 text-center min-h-[60px] flex flex-col justify-center ${colorClass}`}
    >
      <div className={`font-semibold text-sm ${isMuted ? 'text-gray-400' : ''}`}>
        {formatPercent(probability)}
      </div>
      {venue && (
        <div className={`text-xs mt-1 ${isMuted ? 'text-gray-300' : ''}`}>
          <div className="truncate">{venue.name}</div>
          <div className="truncate opacity-75">{venue.city}</div>
        </div>
      )}
    </div>
  );
}
