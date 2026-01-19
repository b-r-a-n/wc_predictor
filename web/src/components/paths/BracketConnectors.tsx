import { useMemo } from 'react';
import type { ConnectorPath, SlotPosition } from './bracketLayout';
import { generateAllConnectors } from './bracketLayout';

interface BracketConnectorsProps {
  positions: SlotPosition[];
  width: number;
  height: number;
  highlightedSlots?: Set<string>;  // Set of "round-slot" keys that are highlighted
  probabilityThreshold?: number;   // Only highlight connectors with probability above this
}

// Generate a unique key for a slot
function slotKey(round: number, slot: number): string {
  return `${round}-${slot}`;
}

export function BracketConnectors({
  positions,
  width,
  height,
  highlightedSlots,
}: BracketConnectorsProps) {
  const connectors = useMemo(
    () => generateAllConnectors(positions),
    [positions]
  );

  // Determine if a connector should be highlighted
  const isConnectorHighlighted = (connector: ConnectorPath): boolean => {
    if (!highlightedSlots || highlightedSlots.size === 0) return false;

    // A connector is highlighted if both its from and to slots are highlighted
    const fromKey = slotKey(connector.fromRound, connector.fromSlot);
    const toKey = slotKey(connector.toRound, connector.toSlot);

    return highlightedSlots.has(fromKey) && highlightedSlots.has(toKey);
  };

  return (
    <svg
      className="absolute inset-0 pointer-events-none"
      width={width}
      height={height}
      style={{ zIndex: 0 }}
    >
      {connectors.map((connector) => {
        const highlighted = isConnectorHighlighted(connector);
        return (
          <path
            key={connector.id}
            d={connector.d}
            fill="none"
            stroke={highlighted ? '#3B82F6' : '#D1D5DB'}
            strokeWidth={highlighted ? 2 : 1}
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        );
      })}
    </svg>
  );
}
