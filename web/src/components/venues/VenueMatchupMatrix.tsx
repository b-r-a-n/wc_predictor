import type { MatchupMatrix } from '../../utils/venueUtils';
import { getFlagEmoji, formatPercent } from '../../utils/formatting';

interface VenueMatchupMatrixProps {
  data: MatchupMatrix;
}

export function VenueMatchupMatrix({ data }: VenueMatchupMatrixProps) {
  const { teams, matrix } = data;

  // Compute max off-diagonal value for heat intensity
  let maxProb = 0;
  for (let i = 0; i < matrix.length; i++) {
    for (let j = 0; j < matrix.length; j++) {
      if (i !== j && matrix[i][j] > maxProb) maxProb = matrix[i][j];
    }
  }

  const cellStyle = (p: number): React.CSSProperties => {
    if (p <= 0 || maxProb === 0) return {};
    const intensity = Math.min(1, p / maxProb);
    return { backgroundColor: `rgba(37, 99, 235, ${intensity * 0.35})` };
  };

  return (
    <div className="space-y-2">
      <div className="text-xs text-gray-500 font-medium">Matchup Probability Matrix</div>
      <div className="overflow-x-auto">
        <table className="text-xs border-collapse">
          <thead>
            <tr>
              <th className="p-1"></th>
              {teams.map((t) => (
                <th key={t.id} className="p-1 font-normal text-gray-600" title={t.name}>
                  <div className="flex flex-col items-center">
                    <span className="text-base leading-none">{getFlagEmoji(t.code)}</span>
                    <span className="text-[10px] uppercase tracking-tight">{t.code}</span>
                  </div>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {teams.map((rowTeam, i) => (
              <tr key={rowTeam.id}>
                <th
                  className="p-1 font-normal text-right text-gray-700 whitespace-nowrap"
                  title={rowTeam.name}
                >
                  <span className="mr-1">{getFlagEmoji(rowTeam.code)}</span>
                  <span className="text-[10px] uppercase tracking-tight">{rowTeam.code}</span>
                </th>
                {teams.map((colTeam, j) => {
                  if (i === j) {
                    return <td key={colTeam.id} className="p-1 bg-gray-100" />;
                  }
                  const p = matrix[i][j];
                  return (
                    <td
                      key={colTeam.id}
                      className="p-1 text-center text-gray-800 border border-gray-100"
                      style={cellStyle(p)}
                      title={`${rowTeam.name} vs ${colTeam.name}`}
                    >
                      {p >= 0.001 ? formatPercent(p, 1) : <span className="text-gray-300">·</span>}
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
