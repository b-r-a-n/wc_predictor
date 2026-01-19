// Bracket layout calculation utilities
// Calculates positions for bracket slots and generates SVG connector paths

export interface SlotPosition {
  round: number;  // 0=R32, 1=R16, 2=QF, 3=SF, 4=Final
  slot: number;
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface ConnectorPath {
  id: string;
  d: string;  // SVG path data
  fromRound: number;
  fromSlot: number;
  toRound: number;
  toSlot: number;
}

export interface LayoutConfig {
  slotWidth: number;
  slotHeight: number;
  roundGap: number;      // Horizontal gap between rounds
  verticalPadding: number;
  baseSlotGap: number;   // Minimum vertical gap between R32 slots
  headerOffset: number;  // Space for round headers at top
}

const DEFAULT_CONFIG: LayoutConfig = {
  slotWidth: 130,
  slotHeight: 60,
  roundGap: 40,
  verticalPadding: 20,
  baseSlotGap: 12,       // Increased from 8 to prevent overlap
  headerOffset: 40,      // Space for round headers
};

// Round configuration
const ROUND_SLOTS = [16, 8, 4, 2, 1];  // R32, R16, QF, SF, Final

// Calculate all slot positions for the bracket
export function calculateSlotPositions(config: Partial<LayoutConfig> = {}): SlotPosition[] {
  const cfg = { ...DEFAULT_CONFIG, ...config };
  const positions: SlotPosition[] = [];

  // For each round
  for (let round = 0; round < ROUND_SLOTS.length; round++) {
    const slotCount = ROUND_SLOTS[round];
    const x = cfg.verticalPadding + round * (cfg.slotWidth + cfg.roundGap);

    if (round === 0) {
      // R32: evenly spaced, offset by header height
      for (let slot = 0; slot < slotCount; slot++) {
        const y = cfg.headerOffset + cfg.verticalPadding + slot * (cfg.slotHeight + cfg.baseSlotGap);
        positions.push({
          round,
          slot,
          x,
          y,
          width: cfg.slotWidth,
          height: cfg.slotHeight,
        });
      }
    } else {
      // Later rounds: center between the two slots they connect
      for (let slot = 0; slot < slotCount; slot++) {
        // Find the two slots from the previous round that feed into this one
        const prevSlot1 = slot * 2;
        const prevSlot2 = slot * 2 + 1;
        const prev1 = positions.find(p => p.round === round - 1 && p.slot === prevSlot1);
        const prev2 = positions.find(p => p.round === round - 1 && p.slot === prevSlot2);

        if (prev1 && prev2) {
          // Center vertically between the two previous slots
          const y1Center = prev1.y + prev1.height / 2;
          const y2Center = prev2.y + prev2.height / 2;
          const y = (y1Center + y2Center) / 2 - cfg.slotHeight / 2;

          positions.push({
            round,
            slot,
            x,
            y,
            width: cfg.slotWidth,
            height: cfg.slotHeight,
          });
        }
      }
    }
  }

  return positions;
}

// Generate SVG path for a bracket connector
export function generateConnectorPath(
  from: SlotPosition,
  to: SlotPosition,
  config: Partial<LayoutConfig> = {}
): string {
  const cfg = { ...DEFAULT_CONFIG, ...config };

  // From the right edge of the "from" slot
  const startX = from.x + from.width;
  const startY = from.y + from.height / 2;

  // To the left edge of the "to" slot
  const endX = to.x;
  const endY = to.y + to.height / 2;

  // Midpoint for the horizontal segment
  const midX = startX + cfg.roundGap / 2;

  // Create a path that goes:
  // 1. Horizontal from start to midpoint
  // 2. Vertical from start Y to end Y
  // 3. Horizontal from midpoint to end
  return `M ${startX} ${startY} H ${midX} V ${endY} H ${endX}`;
}

// Generate all connector paths for the bracket
export function generateAllConnectors(
  positions: SlotPosition[],
  config: Partial<LayoutConfig> = {}
): ConnectorPath[] {
  const connectors: ConnectorPath[] = [];

  // For each round except R32, generate connectors from previous round
  for (let round = 1; round < ROUND_SLOTS.length; round++) {
    const roundSlots = positions.filter(p => p.round === round);

    for (const toSlot of roundSlots) {
      // Two slots from previous round feed into this one
      const fromSlot1 = toSlot.slot * 2;
      const fromSlot2 = toSlot.slot * 2 + 1;

      const from1 = positions.find(p => p.round === round - 1 && p.slot === fromSlot1);
      const from2 = positions.find(p => p.round === round - 1 && p.slot === fromSlot2);

      if (from1) {
        connectors.push({
          id: `r${round - 1}s${fromSlot1}-r${round}s${toSlot.slot}`,
          d: generateConnectorPath(from1, toSlot, config),
          fromRound: round - 1,
          fromSlot: fromSlot1,
          toRound: round,
          toSlot: toSlot.slot,
        });
      }

      if (from2) {
        connectors.push({
          id: `r${round - 1}s${fromSlot2}-r${round}s${toSlot.slot}`,
          d: generateConnectorPath(from2, toSlot, config),
          fromRound: round - 1,
          fromSlot: fromSlot2,
          toRound: round,
          toSlot: toSlot.slot,
        });
      }
    }
  }

  return connectors;
}

// Calculate total bracket dimensions
export function calculateBracketDimensions(
  positions: SlotPosition[],
  config: Partial<LayoutConfig> = {}
): { width: number; height: number } {
  const cfg = { ...DEFAULT_CONFIG, ...config };

  if (positions.length === 0) {
    return { width: 0, height: 0 };
  }

  const maxX = Math.max(...positions.map(p => p.x + p.width));
  const maxY = Math.max(...positions.map(p => p.y + p.height));

  return {
    width: maxX + cfg.verticalPadding + cfg.roundGap, // Extra padding for right side
    height: maxY + cfg.verticalPadding,
  };
}

// Get the round index from round key
export function getRoundIndex(roundKey: string): number {
  const mapping: Record<string, number> = {
    round_of_32: 0,
    round_of_16: 1,
    quarter_finals: 2,
    semi_finals: 3,
    final_match: 4,
  };
  return mapping[roundKey] ?? -1;
}

// Get the round key from round index
export function getRoundKey(roundIndex: number): string {
  const keys = ['round_of_32', 'round_of_16', 'quarter_finals', 'semi_finals', 'final_match'];
  return keys[roundIndex] ?? 'unknown';
}
