import { useMemo } from 'react';
import { formatPercent, getFlagEmoji } from '../../utils/formatting';
import { R32_SLOT_SOURCES, getSlotSourceLabel } from './slotMapping';
import { getMatchForSlot } from '../../utils/matchMapping';
import type {
  Team,
  Group,
  Venue,
  BracketSlotStats,
  SlotOpponentStats,
  TeamStatistics,
  MatchScheduleData,
  KnockoutRoundType,
} from '../../types';

interface TeamPathTreeProps {
  team: Team;
  teamStats: TeamStatistics | undefined;
  groups: Group[];
  bracketStats: BracketSlotStats;
  slotOpponentStats: SlotOpponentStats | null;
  totalSimulations: number;
  teamMap: Map<number, Team>;
  venueMap: Map<string, Venue>;
  schedule: MatchScheduleData | null;
}

const MIN_PROB = 0.005; // hide nodes below 0.5%
const MAX_OPPONENTS = 3;

type RoundKey = 'round_of_32' | 'round_of_16' | 'quarter_finals' | 'semi_finals' | 'final_match';

const LATER_ROUNDS: { key: RoundKey; display: string; mapping: KnockoutRoundType; slotCount: number }[] = [
  { key: 'round_of_32', display: 'Round of 32', mapping: 'round_of_32', slotCount: 16 },
  { key: 'round_of_16', display: 'Round of 16', mapping: 'round_of_16', slotCount: 8 },
  { key: 'quarter_finals', display: 'Quarter-Finals', mapping: 'quarter_finals', slotCount: 4 },
  { key: 'semi_finals', display: 'Semi-Finals', mapping: 'semi_finals', slotCount: 2 },
  { key: 'final_match', display: 'Final', mapping: 'final', slotCount: 1 },
];

function probColor(p: number): string {
  if (p >= 0.2) return 'bg-green-500 text-white border-green-600';
  if (p >= 0.1) return 'bg-green-200 text-green-900 border-green-300';
  if (p >= 0.05) return 'bg-yellow-200 text-yellow-900 border-yellow-300';
  if (p >= 0.01) return 'bg-gray-100 text-gray-700 border-gray-200';
  return 'bg-gray-50 text-gray-500 border-gray-200';
}

function findTeamGroup(teamId: number, groups: Group[]): Group | null {
  return groups.find((g) => g.teams.includes(teamId)) ?? null;
}

// For the selected team's group, figure out which R32 slot each position
// (1st / 2nd / 3rd) feeds into.
function r32SlotsForGroupPosition(group: string, position: 1 | 2 | 3): number[] {
  const slots: number[] = [];
  for (const [slotStr, src] of Object.entries(R32_SLOT_SOURCES)) {
    const slotIdx = Number(slotStr);
    if (position === 3) {
      // Third-place teams feed any pool that includes this group
      // We don't have an explicit pool→groups mapping in TS, so we include
      // every third_place_pool slot. The actual bracket stats will zero out
      // impossible placements.
      if (
        src.teamA.type === 'third_place_pool' ||
        src.teamB.type === 'third_place_pool'
      ) {
        slots.push(slotIdx);
      }
    } else {
      const matches = (side: typeof src.teamA) =>
        side.type === 'group_position' && side.group === group && side.position === position;
      if (matches(src.teamA) || matches(src.teamB)) slots.push(slotIdx);
    }
  }
  return slots;
}

// Label describing how team enters a given R32 slot (from its group's perspective).
function r32EntryLabel(slotIdx: number, teamGroup: string): string | null {
  const src = R32_SLOT_SOURCES[slotIdx];
  if (!src) return null;

  const sideFor = (side: typeof src.teamA): string | null => {
    if (side.type === 'group_position' && side.group === teamGroup) {
      return side.position === 1 ? `1st in ${teamGroup}` : side.position === 2 ? `2nd in ${teamGroup}` : `3rd in ${teamGroup}`;
    }
    if (side.type === 'third_place_pool') {
      return `3rd-place qualifier (Pool ${side.poolIndex! + 1})`;
    }
    return null;
  };

  const a = sideFor(src.teamA);
  const b = sideFor(src.teamB);
  const opponentSide = a ? src.teamB : src.teamA;
  const entry = a ?? b;
  if (!entry) return null;
  return `${entry} vs ${getSlotSourceLabel(opponentSide)}`;
}

interface SlotNodeProps {
  title: string;
  subtitle?: string | null;
  paths?: string[][]; // lineage breadcrumbs — each inner array is one route
  probability: number;
  venueCity?: string;
  opponents: { team: Team; probability: number }[];
}

function SlotNode({ title, subtitle, paths, probability, venueCity, opponents }: SlotNodeProps) {
  return (
    <div className={`w-64 rounded-lg border p-3 shadow-sm ${probColor(probability)}`}>
      <div className="flex items-baseline justify-between gap-2">
        <div className="text-sm font-semibold truncate">{title}</div>
        <div className="text-lg font-bold tabular-nums">{formatPercent(probability)}</div>
      </div>
      {subtitle && <div className="text-xs opacity-75 mt-0.5 truncate">{subtitle}</div>}
      {paths && paths.length > 0 && (
        <div className="mt-1.5 space-y-1">
          {paths.map((p, i) => (
            <div key={i} className="flex items-center gap-0.5 flex-wrap" title={p.join(' → ')}>
              {p.map((hop, j) => {
                const isLast = j === p.length - 1;
                return (
                  <span key={j} className="flex items-center gap-0.5">
                    <span
                      className={`inline-block px-1.5 py-0.5 rounded text-[10px] font-mono leading-tight ${
                        isLast
                          ? 'bg-white/90 text-gray-900 font-semibold ring-1 ring-current/20'
                          : 'bg-black/15 text-current'
                      }`}
                    >
                      {hop}
                    </span>
                    {!isLast && <span className="text-[10px] opacity-50">▸</span>}
                  </span>
                );
              })}
            </div>
          ))}
        </div>
      )}
      {venueCity && <div className="text-[10px] opacity-60 mt-0.5">📍 {venueCity}</div>}
      {opponents.length > 0 && (
        <div className="mt-2 pt-2 border-t border-current/20 space-y-0.5">
          <div className="text-[10px] font-medium opacity-75 uppercase tracking-wide">
            Likely opponents
          </div>
          {opponents.slice(0, MAX_OPPONENTS).map((o) => (
            <div key={o.team.id} className="flex items-center justify-between text-xs">
              <span className="truncate">
                {getFlagEmoji(o.team.code)} {o.team.name}
              </span>
              <span className="tabular-nums opacity-75 ml-1">
                {formatPercent(o.probability / probability, 0)}
              </span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function DownArrow() {
  return (
    <div className="flex justify-center py-1 text-gray-300" aria-hidden>
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5">
        <path d="M12 5 V 19 M5 13 L 12 20 L 19 13" strokeLinecap="round" strokeLinejoin="round" />
      </svg>
    </div>
  );
}

function RoundHeader({ title }: { title: string }) {
  return (
    <div className="text-center">
      <div className="inline-block px-4 py-1 rounded-full bg-gray-800 text-white text-xs font-semibold tracking-wider uppercase">
        {title}
      </div>
    </div>
  );
}

export function TeamPathTree({
  team,
  teamStats,
  groups,
  bracketStats,
  slotOpponentStats,
  totalSimulations,
  teamMap,
  venueMap,
  schedule,
}: TeamPathTreeProps) {
  const group = useMemo(() => findTeamGroup(team.id, groups), [team.id, groups]);
  const groupLetter = group ? group.id.replace(/^Group\s*/i, '') : '?';

  const getVenueCity = (mapping: KnockoutRoundType, slotIdx: number): string | undefined => {
    if (!schedule) return undefined;
    const matchNum = getMatchForSlot(mapping, slotIdx);
    if (!matchNum) return undefined;
    const match = schedule.matches.find((m) => m.matchNumber === matchNum);
    if (!match) return undefined;
    return venueMap.get(match.venueId)?.city;
  };

  const getOpponents = (
    round: RoundKey,
    slotIdx: number
  ): { team: Team; probability: number }[] => {
    if (!slotOpponentStats) return [];
    let opponentCounts: Record<string, number> | undefined;
    if (round === 'final_match') {
      opponentCounts = slotOpponentStats.final_match;
    } else {
      const roundData = slotOpponentStats[round] as Record<string, Record<string, number>> | undefined;
      opponentCounts = roundData?.[String(slotIdx)];
    }
    if (!opponentCounts) return [];
    return Object.entries(opponentCounts)
      .map(([teamIdStr, count]) => ({
        team: teamMap.get(Number(teamIdStr)),
        probability: (count as number) / totalSimulations,
      }))
      .filter((o): o is { team: Team; probability: number } => !!o.team)
      .sort((a, b) => b.probability - a.probability);
  };

  // Group stage outcomes
  const groupOutcomes = useMemo(() => {
    if (!teamStats) return [];
    return [
      {
        key: '1st',
        label: `1st in Group ${groupLetter}`,
        prob: teamStats.group_wins / totalSimulations,
        position: 1 as const,
      },
      {
        key: '2nd',
        label: `2nd in Group ${groupLetter}`,
        prob: teamStats.group_second / totalSimulations,
        position: 2 as const,
      },
      {
        key: '3rd',
        label: `3rd (qualified)`,
        prob: teamStats.group_third_qualified / totalSimulations,
        position: 3 as const,
      },
      {
        key: 'out',
        label: `Eliminated in group`,
        prob: teamStats.group_eliminated / totalSimulations,
        position: null,
      },
    ];
  }, [teamStats, totalSimulations, groupLetter]);

  // Reachable slots per round (prob >= MIN_PROB)
  const reachablePerRound = useMemo(() => {
    const out: Record<RoundKey, number[]> = {
      round_of_32: [],
      round_of_16: [],
      quarter_finals: [],
      semi_finals: [],
      final_match: [],
    };
    for (const r of LATER_ROUNDS) {
      if (r.key === 'final_match') {
        const count = bracketStats.final_match as number;
        if (count / totalSimulations >= MIN_PROB) out.final_match.push(0);
      } else {
        const data = bracketStats[r.key] as Record<string, number>;
        if (!data) continue;
        for (let i = 0; i < r.slotCount; i++) {
          const c = data[String(i)] || 0;
          if (c / totalSimulations >= MIN_PROB) out[r.key].push(i);
        }
      }
    }
    return out;
  }, [bracketStats, totalSimulations]);

  const getSlotProb = (round: RoundKey, slotIdx: number): number => {
    if (round === 'final_match') return (bracketStats.final_match as number) / totalSimulations;
    const data = bracketStats[round] as Record<string, number>;
    return (data?.[String(slotIdx)] || 0) / totalSimulations;
  };

  // Connect group outcome → R32: compute a set of R32 slots per position (for labeling)
  const r32ByPosition = useMemo(() => {
    if (!group) return { 1: [], 2: [], 3: [] } as Record<1 | 2 | 3, number[]>;
    return {
      1: r32SlotsForGroupPosition(groupLetter, 1),
      2: r32SlotsForGroupPosition(groupLetter, 2),
      3: r32SlotsForGroupPosition(groupLetter, 3),
    };
  }, [group, groupLetter]);

  // How does each reachable R32 slot tie back to a group position?
  const r32ToPosition = useMemo(() => {
    const m = new Map<number, 1 | 2 | 3 | null>();
    for (const slot of reachablePerRound.round_of_32) {
      let pos: 1 | 2 | 3 | null = null;
      for (const p of [1, 2, 3] as const) {
        if (r32ByPosition[p].includes(slot)) {
          pos = p;
          break;
        }
      }
      m.set(slot, pos);
    }
    return m;
  }, [reachablePerRound.round_of_32, r32ByPosition]);

  // Short origin label for each reachable R32 slot, from this team's perspective.
  const r32OriginLabel = useMemo(() => {
    const m = new Map<number, string>();
    for (const slot of reachablePerRound.round_of_32) {
      const pos = r32ToPosition.get(slot);
      const label =
        pos === 1 ? `1${groupLetter}`
        : pos === 2 ? `2${groupLetter}`
        : pos === 3 ? `3${groupLetter}`
        : `R32#${slot + 1}`;
      m.set(slot, label);
    }
    return m;
  }, [reachablePerRound.round_of_32, r32ToPosition, groupLetter]);

  // Build lineage paths for every reachable slot in every round.
  // Each lineage is an array of short hop strings ending at that slot.
  // e.g. R16 slot 1 via R32 slot 3 (as 2A): ["2A", "R32#3", "R16#2"]
  const lineages = useMemo(() => {
    const out: Record<RoundKey, Map<number, string[][]>> = {
      round_of_32: new Map(),
      round_of_16: new Map(),
      quarter_finals: new Map(),
      semi_finals: new Map(),
      final_match: new Map(),
    };

    for (const slot of reachablePerRound.round_of_32) {
      const origin = r32OriginLabel.get(slot) ?? `R32#${slot + 1}`;
      out.round_of_32.set(slot, [[origin, `R32#${slot + 1}`]]);
    }

    const cascadeOrder: { round: RoundKey; parent: RoundKey; label: (s: number) => string }[] = [
      { round: 'round_of_16', parent: 'round_of_32', label: (s) => `R16#${s + 1}` },
      { round: 'quarter_finals', parent: 'round_of_16', label: (s) => `QF#${s + 1}` },
      { round: 'semi_finals', parent: 'quarter_finals', label: (s) => `SF#${s + 1}` },
      { round: 'final_match', parent: 'semi_finals', label: () => 'Final' },
    ];

    for (const { round, parent, label } of cascadeOrder) {
      for (const slot of reachablePerRound[round]) {
        const parentSlots = [slot * 2, slot * 2 + 1];
        const paths: string[][] = [];
        for (const ps of parentSlots) {
          const parentPaths = out[parent].get(ps);
          if (parentPaths) {
            for (const pp of parentPaths) {
              paths.push([...pp, label(slot)]);
            }
          }
        }
        if (paths.length === 0) {
          // No reachable parent — shouldn't happen, but fall back to self.
          paths.push([label(slot)]);
        }
        out[round].set(slot, paths);
      }
    }
    return out;
  }, [reachablePerRound, r32OriginLabel]);

  return (
    <div className="bg-white rounded-lg border border-gray-200 p-5 space-y-2 max-w-3xl mx-auto">
      <div className="text-xs text-gray-500 text-center">
        Possible paths for {team.name}. Only outcomes with probability ≥ {(MIN_PROB * 100).toFixed(1)}% are shown.
        Opponent percentages are conditional on reaching that slot.
      </div>

      {/* Group stage */}
      <RoundHeader title={`Group Stage — Group ${groupLetter}`} />
      <div className="flex flex-wrap justify-center gap-3">
        {groupOutcomes.map((o) => {
          const visible = o.prob >= MIN_PROB;
          if (!visible) return null;
          const eliminated = o.position === null;
          return (
            <div
              key={o.key}
              className={`w-48 rounded-lg border p-3 shadow-sm ${
                eliminated ? 'bg-red-50 text-red-700 border-red-200' : probColor(o.prob)
              }`}
            >
              <div className="flex items-baseline justify-between gap-2">
                <div className="text-sm font-semibold">{o.label}</div>
                <div className="text-lg font-bold tabular-nums">{formatPercent(o.prob)}</div>
              </div>
              {eliminated && <div className="text-xs opacity-75 mt-1">Tournament ends here</div>}
            </div>
          );
        })}
      </div>

      <DownArrow />

      {/* Later rounds */}
      {LATER_ROUNDS.map((r, idx) => {
        const slots = reachablePerRound[r.key];
        if (slots.length === 0) {
          return (
            <div key={r.key}>
              <RoundHeader title={r.display} />
              <div className="text-center text-sm text-gray-400 py-2">
                No realistic path to this round.
              </div>
            </div>
          );
        }
        return (
          <div key={r.key}>
            <RoundHeader title={r.display} />
            <div className="flex flex-wrap justify-center gap-3 mt-2">
              {slots.map((slot) => {
                const prob = getSlotProb(r.key, slot);
                const opponents = getOpponents(r.key, slot);
                const venueCity = getVenueCity(r.mapping, slot);
                const slotLineages = lineages[r.key].get(slot) ?? [];
                let title: string;
                let subtitle: string | null = null;

                if (r.key === 'round_of_32') {
                  const pos = r32ToPosition.get(slot) ?? null;
                  const entryLabel = r32EntryLabel(slot, groupLetter);
                  title = pos === 1
                    ? `R32 #${slot + 1} — as 1${groupLetter}`
                    : pos === 2
                    ? `R32 #${slot + 1} — as 2${groupLetter}`
                    : pos === 3
                    ? `R32 #${slot + 1} — as 3${groupLetter}`
                    : `R32 Slot ${slot + 1}`;
                  subtitle = entryLabel;
                } else if (r.key === 'final_match') {
                  title = 'Final';
                } else {
                  title = `${r.display} — Slot ${slot + 1}`;
                }

                return (
                  <SlotNode
                    key={`${r.key}-${slot}`}
                    title={title}
                    subtitle={subtitle}
                    paths={slotLineages}
                    probability={prob}
                    venueCity={venueCity}
                    opponents={opponents}
                  />
                );
              })}
            </div>
            {idx < LATER_ROUNDS.length - 1 && <DownArrow />}
          </div>
        );
      })}
    </div>
  );
}
