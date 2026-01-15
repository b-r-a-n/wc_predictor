import type { Team, TeamStatistics } from '../../types';
import { TeamSlot } from './TeamSlot';

interface BracketMatchProps {
  team1: Team | null;
  team2: Team | null;
  stats1?: TeamStatistics;
  stats2?: TeamStatistics;
  totalSimulations: number;
  roundKey: 'reached_round_of_32' | 'reached_round_of_16' | 'reached_quarter_finals' | 'reached_semi_finals' | 'reached_final';
}

export function BracketMatch({
  team1,
  team2,
  stats1,
  stats2,
  totalSimulations,
  roundKey,
}: BracketMatchProps) {
  const prob1 =
    stats1 && totalSimulations > 0 ? stats1[roundKey] / totalSimulations : 0;
  const prob2 =
    stats2 && totalSimulations > 0 ? stats2[roundKey] / totalSimulations : 0;

  const winner = prob1 >= prob2 ? 1 : 2;

  return (
    <div className="flex flex-col gap-1">
      <TeamSlot
        team={team1}
        probability={prob1}
        isWinner={winner === 1 && prob1 > 0}
        size="sm"
      />
      <TeamSlot
        team={team2}
        probability={prob2}
        isWinner={winner === 2 && prob2 > 0}
        size="sm"
      />
    </div>
  );
}
