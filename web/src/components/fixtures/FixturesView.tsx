import { useMemo } from 'react';
import { useSimulatorStore, effectiveFixedResults } from '../../store/simulatorStore';
import { getFlagEmoji } from '../../utils/formatting';
import { resolveGroupMatchTeams } from '../../utils/fixedResults';
import { Button } from '../common';
import type { Team, ScheduledMatch, FixedMatchResult } from '../../types';

function fixedResultsEqual(
  a: Record<number, FixedMatchResult>,
  b: Record<number, FixedMatchResult>
): boolean {
  const aKeys = Object.keys(a);
  const bKeys = Object.keys(b);
  if (aKeys.length !== bKeys.length) return false;
  for (const key of aKeys) {
    const ai = a[Number(key)];
    const bi = b[Number(key)];
    if (!bi) return false;
    if (ai.homeScore !== bi.homeScore || ai.awayScore !== bi.awayScore) return false;
  }
  return true;
}

function formatUpdated(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
}

interface GroupedMatch {
  match: ScheduledMatch;
  home: Team;
  away: Team;
}

export function FixturesView() {
  const {
    schedule,
    groups,
    teams,
    fixedResults,
    actualResults,
    actualResultsInfo,
    useActualResults,
    setUseActualResults,
    lastSimulatedFixedResults,
    setFixedResult,
    clearFixedResult,
    clearAllFixedResults,
    runSimulation,
    isSimulating,
    wasmStatus,
  } = useSimulatorStore();

  const effective = useMemo(
    () => effectiveFixedResults(useActualResults, actualResults, fixedResults),
    [useActualResults, actualResults, fixedResults]
  );

  const isDirty = useMemo(() => {
    const baseline = lastSimulatedFixedResults ?? {};
    return !fixedResultsEqual(effective, baseline);
  }, [effective, lastSimulatedFixedResults]);

  const actualCount = actualResultsInfo?.count ?? 0;

  const canRun = wasmStatus === 'ready' && !isSimulating;

  const teamMap = useMemo(() => {
    const m = new Map<number, Team>();
    teams.forEach((t) => m.set(t.id, t));
    return m;
  }, [teams]);

  const groupedMatches = useMemo(() => {
    const byGroup = new Map<string, GroupedMatch[]>();
    if (!schedule) return byGroup;
    for (const match of schedule.matches) {
      if (match.round !== 'group_stage' || !match.groupId) continue;
      const resolved = resolveGroupMatchTeams(match, groups);
      if (!resolved) continue;
      const home = teamMap.get(resolved.homeTeamId);
      const away = teamMap.get(resolved.awayTeamId);
      if (!home || !away) continue;
      if (!byGroup.has(match.groupId)) byGroup.set(match.groupId, []);
      byGroup.get(match.groupId)!.push({ match, home, away });
    }
    for (const list of byGroup.values()) {
      list.sort((a, b) => a.match.matchNumber - b.match.matchNumber);
    }
    return byGroup;
  }, [schedule, groups, teamMap]);

  const lockedCount = Object.keys(fixedResults).length;

  if (!schedule) {
    return (
      <div className="flex items-center justify-center h-64 text-gray-500">
        Loading match schedule…
      </div>
    );
  }

  const groupIds = Array.from(groupedMatches.keys()).sort();

  return (
    <div className="space-y-4">
      <div className="flex flex-col sm:flex-row sm:items-end sm:justify-between gap-2">
        <div>
          <h2 className="text-lg font-semibold text-gray-900">Group-Stage Results</h2>
          <p className="text-sm text-gray-500">
            Real results are pinned automatically; everything else is simulated. Enter a score to
            override a match with a hypothetical result. Re-run the simulation to apply changes.
          </p>
        </div>
        <div className="flex items-center gap-3">
          <span className="text-sm text-gray-600">
            {lockedCount === 0
              ? 'No overrides'
              : `${lockedCount} override${lockedCount === 1 ? '' : 's'}`}
          </span>
          <Button
            variant="secondary"
            size="sm"
            onClick={clearAllFixedResults}
            disabled={lockedCount === 0}
          >
            Clear overrides
          </Button>
        </div>
      </div>

      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 rounded-md border border-gray-200 bg-gray-50 px-4 py-3">
        <label className="flex items-center gap-3 cursor-pointer select-none">
          <input
            type="checkbox"
            className="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
            checked={useActualResults}
            onChange={(e) => setUseActualResults(e.target.checked)}
          />
          <span className="text-sm font-medium text-gray-900">
            Start from real results
          </span>
        </label>
        <span className="text-sm text-gray-500">
          {actualCount === 0
            ? 'No real results loaded yet'
            : `${actualCount} match${actualCount === 1 ? '' : 'es'} played` +
              (actualResultsInfo?.generated_at
                ? ` · updated ${formatUpdated(actualResultsInfo.generated_at)}`
                : '')}
        </span>
      </div>

      {isDirty && (
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 rounded-md border border-amber-300 bg-amber-50 px-4 py-3">
          <div className="flex items-center gap-2 text-sm text-amber-900">
            <svg xmlns="http://www.w3.org/2000/svg" className="w-5 h-5 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
            <span>
              Results are out of date — locked matches have changed since the last simulation.
            </span>
          </div>
          <Button
            size="sm"
            onClick={runSimulation}
            disabled={!canRun}
          >
            {isSimulating ? 'Running…' : 'Re-run simulation'}
          </Button>
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
        {groupIds.map((groupId) => {
          const matches = groupedMatches.get(groupId) ?? [];
          return (
            <div key={groupId} className="bg-white rounded-lg border border-gray-200 p-4">
              <h3 className="text-sm font-semibold text-gray-900 mb-3">Group {groupId}</h3>
              <div className="space-y-2">
                {matches.map(({ match, home, away }) => {
                  const override = fixedResults[match.matchNumber];
                  const actual = useActualResults ? actualResults[match.matchNumber] : undefined;
                  // What the simulation will use: override beats real result.
                  const current = override ?? actual;
                  const isOverride = !!override;
                  const isActual = !override && !!actual;
                  return (
                    <div
                      key={match.matchNumber}
                      className={`flex items-center gap-2 p-2 rounded border ${
                        isOverride
                          ? 'border-blue-300 bg-blue-50'
                          : isActual
                          ? 'border-emerald-300 bg-emerald-50'
                          : 'border-gray-200'
                      }`}
                    >
                      <div className="flex-1 text-right text-sm font-medium truncate">
                        <span className="mr-1">{getFlagEmoji(home.code)}</span>
                        {home.name}
                      </div>
                      <ScoreInput
                        value={current?.homeScore ?? null}
                        onChange={(v) => {
                          const awayV = current?.awayScore ?? 0;
                          if (v == null) {
                            clearFixedResult(match.matchNumber);
                          } else {
                            setFixedResult(match.matchNumber, v, awayV);
                          }
                        }}
                      />
                      <span className="text-gray-400 text-xs">vs</span>
                      <ScoreInput
                        value={current?.awayScore ?? null}
                        onChange={(v) => {
                          const homeV = current?.homeScore ?? 0;
                          if (v == null) {
                            clearFixedResult(match.matchNumber);
                          } else {
                            setFixedResult(match.matchNumber, homeV, v);
                          }
                        }}
                      />
                      <div className="flex-1 text-sm font-medium truncate">
                        {away.name}
                        <span className="ml-1">{getFlagEmoji(away.code)}</span>
                      </div>
                      <span
                        className={`text-[10px] font-semibold uppercase tracking-wide w-12 text-center ${
                          isOverride
                            ? 'text-blue-600'
                            : isActual
                            ? 'text-emerald-600'
                            : 'text-transparent'
                        }`}
                      >
                        {isOverride ? 'Edited' : isActual ? 'Played' : '—'}
                      </span>
                      <button
                        onClick={() => clearFixedResult(match.matchNumber)}
                        disabled={!isOverride}
                        className="text-gray-400 hover:text-red-600 disabled:opacity-30 disabled:cursor-not-allowed text-sm px-1"
                        title={isActual ? 'Revert to real result' : 'Clear override'}
                        aria-label="Clear override"
                      >
                        &times;
                      </button>
                    </div>
                  );
                })}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

interface ScoreInputProps {
  value: number | null;
  onChange: (value: number | null) => void;
}

function ScoreInput({ value, onChange }: ScoreInputProps) {
  return (
    <input
      type="number"
      min={0}
      max={20}
      value={value ?? ''}
      onChange={(e) => {
        const raw = e.target.value;
        if (raw === '') {
          onChange(null);
          return;
        }
        const n = parseInt(raw, 10);
        if (!Number.isFinite(n) || n < 0) {
          onChange(null);
          return;
        }
        onChange(Math.min(n, 20));
      }}
      placeholder="—"
      className="w-10 text-center border border-gray-300 rounded px-1 py-0.5 text-sm tabular-nums focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
    />
  );
}
