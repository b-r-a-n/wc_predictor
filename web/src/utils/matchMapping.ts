// Match number to slot mapping utilities
// Derived from bracket.rs R32_BRACKET array order
// Array index in bracket.rs = slot number, match_num field = FIFA match number

/**
 * Maps FIFA R32 match numbers to bracket slot indices.
 * Source of truth: crates/wc-core/src/bracket.rs R32_BRACKET constant.
 */
export const R32_MATCH_TO_SLOT: Record<number, number> = {
  74: 0,  77: 1,  73: 2,  75: 3,
  76: 4,  78: 5,  79: 6,  80: 7,
  83: 8,  84: 9,  81: 10, 82: 11,
  86: 12, 88: 13, 85: 14, 87: 15,
};

/**
 * Maps bracket slot indices to FIFA R32 match numbers.
 */
export const R32_SLOT_TO_MATCH: Record<number, number> = Object.fromEntries(
  Object.entries(R32_MATCH_TO_SLOT).map(([matchNum, slot]) => [slot, Number(matchNum)])
);

/**
 * Maps FIFA R16 match numbers to bracket slot indices.
 * R16 matches 89-96 map to slots 0-7 sequentially.
 */
export const R16_MATCH_TO_SLOT: Record<number, number> = {
  89: 0, 90: 1, 91: 2, 92: 3, 93: 4, 94: 5, 95: 6, 96: 7,
};

/**
 * Maps FIFA QF match numbers to bracket slot indices.
 * QF matches 97-100 map to slots 0-3 sequentially.
 */
export const QF_MATCH_TO_SLOT: Record<number, number> = {
  97: 0, 98: 1, 99: 2, 100: 3,
};

/**
 * Maps FIFA SF match numbers to bracket slot indices.
 * SF matches 101-102 map to slots 0-1 sequentially.
 */
export const SF_MATCH_TO_SLOT: Record<number, number> = {
  101: 0, 102: 1,
};

/** FIFA match number for the final */
export const FINAL_MATCH = 104;

/** FIFA match number for the third place match */
export const THIRD_PLACE_MATCH = 103;

/**
 * Get the bracket slot index for a given match number.
 * Returns undefined if match number is not a knockout match.
 */
export function getSlotForMatch(matchNumber: number): number | undefined {
  return R32_MATCH_TO_SLOT[matchNumber]
    ?? R16_MATCH_TO_SLOT[matchNumber]
    ?? QF_MATCH_TO_SLOT[matchNumber]
    ?? SF_MATCH_TO_SLOT[matchNumber]
    ?? (matchNumber === FINAL_MATCH ? 0 : undefined)
    ?? (matchNumber === THIRD_PLACE_MATCH ? 0 : undefined);
}

/**
 * Get the FIFA match number for a given round and slot index.
 * Returns undefined if the round/slot combination is invalid.
 */
export function getMatchForSlot(round: string, slot: number): number | undefined {
  switch (round) {
    case 'round_of_32':
      return R32_SLOT_TO_MATCH[slot];
    case 'round_of_16':
      return 89 + slot;
    case 'quarter_finals':
      return 97 + slot;
    case 'semi_finals':
      return 101 + slot;
    case 'final':
      return FINAL_MATCH;
    case 'third_place':
      return THIRD_PLACE_MATCH;
    default:
      return undefined;
  }
}
