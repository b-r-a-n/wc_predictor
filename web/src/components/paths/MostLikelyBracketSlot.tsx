import { formatPercent, getFlagEmoji } from '../../utils/formatting';
import { BracketTooltip } from './BracketTooltip';
import { getSlotSourceDescription, R32_SLOT_SOURCES, ROUND_DISPLAY_NAMES } from './slotMapping';
import type { Venue, MostLikelySlotData } from '../../types';

interface MostLikelyBracketSlotProps {
  round: string;           // "round_of_32", "round_of_16", etc.
  slotIndex: number;       // 0-15 for R32, 0-7 for R16, etc.
  slotData: MostLikelySlotData | null;
  venue?: Venue;
  isWinnerPath?: boolean;  // Whether this slot is part of the winner's path
  onTeamClick?: (teamId: number) => void;
  style?: React.CSSProperties;
}

// Get background color class based on probability with winner path override
function getSlotColorClass(probability: number, isWinnerPath: boolean): string {
  if (isWinnerPath) {
    // Amber/gold for winner's path
    return 'bg-amber-400 text-amber-900 ring-2 ring-amber-500';
  }

  if (probability > 0.20) {
    return 'bg-blue-500 text-white';
  } else if (probability >= 0.10) {
    return 'bg-blue-300 text-gray-900';
  } else if (probability >= 0.05) {
    return 'bg-blue-200 text-gray-900';
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

export function MostLikelyBracketSlot({
  round,
  slotIndex,
  slotData,
  venue,
  isWinnerPath = false,
  onTeamClick,
  style,
}: MostLikelyBracketSlotProps) {
  const probability = slotData?.probability ?? 0;
  const colorClass = getSlotColorClass(probability, isWinnerPath);
  const isEmpty = !slotData;
  const isR32 = round === 'round_of_32';
  const roundName = ROUND_DISPLAY_NAMES[round] || round;

  const handleClick = () => {
    if (slotData && onTeamClick) {
      onTeamClick(slotData.teamId);
    }
  };

  const slotContent = (
    <div
      className={`rounded-md p-2 text-center flex flex-col justify-center overflow-hidden transition-all ${colorClass} ${
        slotData ? 'cursor-pointer hover:ring-2 hover:ring-blue-400' : ''
      }`}
      style={{
        height: '70px',
        ...style,
      }}
      onClick={handleClick}
    >
      {slotData ? (
        <>
          {/* Team flag and name - no wrap */}
          <div className="flex items-center justify-center gap-1 flex-nowrap min-w-0">
            <span className="text-xs flex-shrink-0">{getFlagEmoji(slotData.team.code)}</span>
            <span className="font-semibold text-xs truncate whitespace-nowrap">
              {slotData.team.name}
            </span>
          </div>
          {/* Probability */}
          <div className={`text-[10px] leading-tight ${isWinnerPath ? 'text-amber-800' : 'opacity-75'}`}>
            {formatPercent(probability)}
          </div>
          {/* Venue */}
          {venue && (
            <div className={`text-[10px] leading-tight ${isWinnerPath ? 'text-amber-700' : 'opacity-60'} truncate`}>
              {venue.city}
            </div>
          )}
        </>
      ) : (
        <div className="text-gray-400 text-xs">
          No data
        </div>
      )}
    </div>
  );

  // Wrap R32 slots with tooltip showing match sources
  if (isR32) {
    return (
      <BracketTooltip
        content={getR32TooltipContent(slotIndex)}
        disabled={isEmpty}
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
          {slotData && (
            <div className="mt-1 text-gray-200">
              Click to view {slotData.team.name}'s bracket
            </div>
          )}
        </div>
      }
      disabled={isEmpty}
    >
      {slotContent}
    </BracketTooltip>
  );
}
