import { useSimulatorStore } from '../../store/simulatorStore';
import { formatNumber } from '../../utils/formatting';

export function BaselineBanner() {
  const { baselineInfo, teamsModified } = useSimulatorStore();

  if (!baselineInfo) return null;

  const date = (() => {
    try {
      return new Date(baselineInfo.generated_at).toLocaleDateString(undefined, {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
      });
    } catch {
      return baselineInfo.generated_at;
    }
  })();

  const strategyLabel =
    baselineInfo.strategy === 'composite' ? 'composite' : baselineInfo.strategy;

  return (
    <div className="rounded-md border border-blue-200 bg-blue-50 px-3 py-2 text-xs text-blue-800 flex flex-wrap items-center gap-2">
      <span className="font-semibold">Pre-simulated baseline</span>
      <span className="opacity-80">
        {formatNumber(baselineInfo.iterations)} iterations · {strategyLabel} · generated {date}
      </span>
      {teamsModified && (
        <span className="text-amber-700 font-medium">— team edits pending, re-run to refresh</span>
      )}
    </div>
  );
}
