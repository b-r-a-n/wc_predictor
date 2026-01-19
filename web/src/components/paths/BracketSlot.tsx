import { formatPercent, getFlagEmoji } from '../../utils/formatting';
import { BracketTooltip } from './BracketTooltip';
import { getSlotSourceDescription, R32_SLOT_SOURCES, ROUND_DISPLAY_NAMES } from './slotMapping';
import type { Venue, Team } from '../../types';

interface OpponentInfo {
  team: Team;
  probability: number;
}

interface BracketSlotProps {
  round: string;           // "round_of_32", "round_of_16", etc.
  slotIndex: number;       // 0-15 for R32, 0-7 for R16, etc.
  probability: number;     // 0.0 to 1.0
  totalSimulations: number;
  venue?: Venue;
  topOpponents?: OpponentInfo[];  // Top opponents for this slot
  isExpanded?: boolean;
  onToggleExpand?: () => void;
  style?: React.CSSProperties;
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

// Get tooltip content for R32 slots
function getR32TooltipContent(slotIndex: number): React.ReactNode {
  const sources = R32_SLOT_SOURCES[slotIndex];
  if (!sources) return null;

  return (
    <div className="text-xs space-y-1">
      <div className="font-medium text-gray-200">Match Sources:</div>
      <div>{getSlotSourceDescription(sources.teamA)}</div>
      <div className="text-gray-400">vs</div>
      <div>{getSlotSourceDescription(sources.teamB)}</div>
    </div>
  );
}

export function BracketSlot({
  round,
  slotIndex,
  probability,
  venue,
  topOpponents,
  isExpanded,
  onToggleExpand,
  style,
}: BracketSlotProps) {
  const colorClass = getProbabilityColorClass(probability);
  const isMuted = probability < 0.01;
  const isR32 = round === 'round_of_32';
  const hasOpponents = topOpponents && topOpponents.length > 0;
  const roundName = ROUND_DISPLAY_NAMES[round] || round;

  const slotContent = (
    <div
      className={`rounded-md p-2 text-center flex flex-col justify-center cursor-pointer transition-all hover:ring-2 hover:ring-blue-400 ${colorClass} ${
        isExpanded ? 'ring-2 ring-blue-500' : ''
      }`}
      style={{
        minHeight: isExpanded ? 'auto' : '60px',
        ...style,
      }}
      onClick={onToggleExpand}
    >
      <div className={`font-semibold text-sm ${isMuted ? 'text-gray-400' : ''}`}>
        {formatPercent(probability)}
      </div>
      {venue && (
        <div className={`text-xs mt-1 ${isMuted ? 'text-gray-300' : 'opacity-75'}`}>
          <div className="truncate">{venue.city}</div>
        </div>
      )}

      {/* Expanded opponent list */}
      {isExpanded && hasOpponents && (
        <div className="mt-2 pt-2 border-t border-current/20 text-xs space-y-1">
          <div className="font-medium opacity-75">Top Opponents:</div>
          {topOpponents.slice(0, 5).map((opp) => (
            <div key={opp.team.id} className="flex items-center justify-between gap-1">
              <span className="truncate">
                {getFlagEmoji(opp.team.code)} {opp.team.name}
              </span>
              <span className="opacity-75">{formatPercent(opp.probability)}</span>
            </div>
          ))}
        </div>
      )}

      {/* Expand indicator */}
      {hasOpponents && !isExpanded && (
        <div className="text-xs opacity-50 mt-1">
          Click to see opponents
        </div>
      )}
    </div>
  );

  // Wrap R32 slots with tooltip showing match sources
  if (isR32) {
    return (
      <BracketTooltip
        content={getR32TooltipContent(slotIndex)}
        disabled={isMuted}
      >
        <div className="relative">
          {slotContent}
          {/* Slot number indicator for R32 */}
          <div className="absolute -left-6 top-1/2 -translate-y-1/2 text-xs text-gray-400">
            #{slotIndex + 1}
          </div>
        </div>
      </BracketTooltip>
    );
  }

  // Later rounds: wrap with tooltip showing round info
  return (
    <BracketTooltip
      content={
        <div className="text-xs">
          <div className="font-medium">{roundName}</div>
          {venue && <div className="text-gray-300">{venue.name}, {venue.city}</div>}
        </div>
      }
      disabled={isMuted}
    >
      {slotContent}
    </BracketTooltip>
  );
}
