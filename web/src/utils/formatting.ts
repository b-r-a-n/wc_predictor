// Formatting utilities

export function formatPercent(value: number, decimals = 1): string {
  return `${(value * 100).toFixed(decimals)}%`;
}

export function formatNumber(value: number): string {
  return new Intl.NumberFormat().format(value);
}

// Get flag emoji from country code
export function getFlagEmoji(countryCode: string): string {
  // Handle special cases
  const codeMap: Record<string, string> = {
    ENG: 'GB-ENG',
    SCO: 'GB-SCT',
    WAL: 'GB-WLS',
  };

  const code = codeMap[countryCode] || countryCode;

  // For UK subdivisions, we'll return text flag indicators instead
  if (code.startsWith('GB-')) {
    const flags: Record<string, string> = {
      'GB-ENG': '\uD83C\uDFF4\uDB40\uDC67\uDB40\uDC62\uDB40\uDC65\uDB40\uDC6E\uDB40\uDC67\uDB40\uDC7F',
      'GB-SCT': '\uD83C\uDFF4\uDB40\uDC67\uDB40\uDC62\uDB40\uDC73\uDB40\uDC63\uDB40\uDC74\uDB40\uDC7F',
      'GB-WLS': '\uD83C\uDFF4\uDB40\uDC67\uDB40\uDC62\uDB40\uDC77\uDB40\uDC6C\uDB40\uDC73\uDB40\uDC7F',
    };
    return flags[code] || '\uD83C\uDFF3\uFE0F';
  }

  // Standard country code to flag emoji conversion
  const codePoints = code
    .toUpperCase()
    .split('')
    .map((char) => 127397 + char.charCodeAt(0));

  return String.fromCodePoint(...codePoints);
}

// Get group letter from index
export function getGroupLetter(index: number): string {
  return String.fromCharCode(65 + index); // A, B, C, ...
}

// Sort teams by win probability
export function sortByWinProbability<T extends { win_probability: number }>(
  teams: T[],
  ascending = false
): T[] {
  return [...teams].sort((a, b) =>
    ascending ? a.win_probability - b.win_probability : b.win_probability - a.win_probability
  );
}
