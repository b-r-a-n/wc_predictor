import type { Strategy } from '../../types';

interface StrategySelectorProps {
  value: Strategy;
  onChange: (strategy: Strategy) => void;
}

const strategies: { value: Strategy; label: string; description: string }[] = [
  { value: 'elo', label: 'ELO Rating', description: 'World Football ELO ratings' },
  { value: 'market_value', label: 'Market Value', description: 'Squad market values' },
  { value: 'fifa_ranking', label: 'FIFA Ranking', description: 'Official FIFA rankings' },
  { value: 'composite', label: 'Composite', description: 'Weighted combination' },
];

export function StrategySelector({ value, onChange }: StrategySelectorProps) {
  return (
    <div className="space-y-2">
      <label className="block text-sm font-medium text-gray-700">Prediction Strategy</label>
      <div className="space-y-2">
        {strategies.map((strategy) => (
          <label
            key={strategy.value}
            className={`flex items-start p-3 rounded-lg border cursor-pointer transition-colors ${
              value === strategy.value
                ? 'border-blue-500 bg-blue-50'
                : 'border-gray-200 hover:border-gray-300'
            }`}
          >
            <input
              type="radio"
              name="strategy"
              value={strategy.value}
              checked={value === strategy.value}
              onChange={() => onChange(strategy.value)}
              className="mt-0.5 h-4 w-4 text-blue-600 focus:ring-blue-500"
            />
            <div className="ml-3">
              <span className="block text-sm font-medium text-gray-900">{strategy.label}</span>
              <span className="block text-xs text-gray-500">{strategy.description}</span>
            </div>
          </label>
        ))}
      </div>
    </div>
  );
}
