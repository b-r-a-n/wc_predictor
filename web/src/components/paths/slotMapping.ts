// R32 slot-to-group position mapping for bracket tooltips
// Based on FIFA World Cup 2026 bracket structure with 12 groups

// Describes the source of each team in a R32 slot
export interface SlotSource {
  teamA: SlotSourceInfo;
  teamB: SlotSourceInfo;
}

export interface SlotSourceInfo {
  type: 'group_position' | 'third_place_pool';
  group?: string;  // e.g., "A", "B", ... "L"
  position?: 1 | 2 | 3;
  poolIndex?: number;  // For third place qualifiers (0-7)
}

// R32 bracket slot mapping
// Each slot represents a match, with teamA from one source and teamB from another
// Source of truth: crates/wc-core/src/bracket.rs R32_BRACKET constant
// Array index in bracket.rs = slot number
export const R32_SLOT_SOURCES: Record<number, SlotSource> = {
  // Slot 0: Match 74 - 1E vs 3rd (A/B/C/D/F)
  0: {
    teamA: { type: 'group_position', group: 'E', position: 1 },
    teamB: { type: 'third_place_pool', poolIndex: 0 },
  },
  // Slot 1: Match 77 - 1I vs 3rd (C/D/F/G/H)
  1: {
    teamA: { type: 'group_position', group: 'I', position: 1 },
    teamB: { type: 'third_place_pool', poolIndex: 1 },
  },
  // Slot 2: Match 73 - 2A vs 2B
  2: {
    teamA: { type: 'group_position', group: 'A', position: 2 },
    teamB: { type: 'group_position', group: 'B', position: 2 },
  },
  // Slot 3: Match 75 - 1F vs 2C
  3: {
    teamA: { type: 'group_position', group: 'F', position: 1 },
    teamB: { type: 'group_position', group: 'C', position: 2 },
  },
  // Slot 4: Match 76 - 1C vs 2F
  4: {
    teamA: { type: 'group_position', group: 'C', position: 1 },
    teamB: { type: 'group_position', group: 'F', position: 2 },
  },
  // Slot 5: Match 78 - 2E vs 2I
  5: {
    teamA: { type: 'group_position', group: 'E', position: 2 },
    teamB: { type: 'group_position', group: 'I', position: 2 },
  },
  // Slot 6: Match 79 - 1A vs 3rd (C/E/F/H/I)
  6: {
    teamA: { type: 'group_position', group: 'A', position: 1 },
    teamB: { type: 'third_place_pool', poolIndex: 2 },
  },
  // Slot 7: Match 80 - 1L vs 3rd (E/H/I/J/K)
  7: {
    teamA: { type: 'group_position', group: 'L', position: 1 },
    teamB: { type: 'third_place_pool', poolIndex: 3 },
  },
  // Slot 8: Match 83 - 2K vs 2L
  8: {
    teamA: { type: 'group_position', group: 'K', position: 2 },
    teamB: { type: 'group_position', group: 'L', position: 2 },
  },
  // Slot 9: Match 84 - 1H vs 2J
  9: {
    teamA: { type: 'group_position', group: 'H', position: 1 },
    teamB: { type: 'group_position', group: 'J', position: 2 },
  },
  // Slot 10: Match 81 - 1D vs 3rd (B/E/F/I/J)
  10: {
    teamA: { type: 'group_position', group: 'D', position: 1 },
    teamB: { type: 'third_place_pool', poolIndex: 4 },
  },
  // Slot 11: Match 82 - 1G vs 3rd (A/E/H/I/J)
  11: {
    teamA: { type: 'group_position', group: 'G', position: 1 },
    teamB: { type: 'third_place_pool', poolIndex: 5 },
  },
  // Slot 12: Match 86 - 1J vs 2H
  12: {
    teamA: { type: 'group_position', group: 'J', position: 1 },
    teamB: { type: 'group_position', group: 'H', position: 2 },
  },
  // Slot 13: Match 88 - 2D vs 2G
  13: {
    teamA: { type: 'group_position', group: 'D', position: 2 },
    teamB: { type: 'group_position', group: 'G', position: 2 },
  },
  // Slot 14: Match 85 - 1B vs 3rd (E/F/G/I/J)
  14: {
    teamA: { type: 'group_position', group: 'B', position: 1 },
    teamB: { type: 'third_place_pool', poolIndex: 6 },
  },
  // Slot 15: Match 87 - 1K vs 3rd (D/E/I/J/L)
  15: {
    teamA: { type: 'group_position', group: 'K', position: 1 },
    teamB: { type: 'third_place_pool', poolIndex: 7 },
  },
};

// Get human-readable description for a slot source
export function getSlotSourceDescription(source: SlotSourceInfo): string {
  if (source.type === 'group_position') {
    const positionText = source.position === 1 ? '1st' : source.position === 2 ? '2nd' : '3rd';
    return `Finishes ${positionText} in Group ${source.group}`;
  } else {
    return `3rd place qualifier (Pool ${source.poolIndex! + 1})`;
  }
}

// Get short label for a slot source (for compact display)
export function getSlotSourceLabel(source: SlotSourceInfo): string {
  if (source.type === 'group_position') {
    return `${source.position}${source.group}`;
  } else {
    return `3rd-${source.poolIndex! + 1}`;
  }
}

// Round display names
export const ROUND_DISPLAY_NAMES: Record<string, string> = {
  round_of_32: 'Round of 32',
  round_of_16: 'Round of 16',
  quarter_finals: 'Quarter Finals',
  semi_finals: 'Semi Finals',
  final_match: 'Final',
};
