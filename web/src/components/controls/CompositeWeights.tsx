import type { CompositeWeights as CompositeWeightsType } from '../../types';

interface CompositeWeightsProps {
  value: CompositeWeightsType;
  onChange: (weights: CompositeWeightsType) => void;
}

export function CompositeWeights({ value, onChange }: CompositeWeightsProps) {
  const handleChange = (key: keyof CompositeWeightsType, newValue: number) => {
    // Normalize weights to sum to 1
    const others = Object.keys(value).filter((k) => k !== key) as (keyof CompositeWeightsType)[];
    const otherSum = others.reduce((sum, k) => sum + value[k], 0);

    if (otherSum === 0) {
      // If other weights are 0, just set this one
      onChange({ ...value, [key]: newValue });
    } else {
      // Scale other weights proportionally
      const remaining = 1 - newValue;
      const scale = remaining / otherSum;
      const newWeights = { ...value, [key]: newValue };
      others.forEach((k) => {
        newWeights[k] = Math.max(0, Math.min(1, value[k] * scale));
      });
      onChange(newWeights);
    }
  };

  const weights: { key: keyof CompositeWeightsType; label: string }[] = [
    { key: 'elo', label: 'ELO Rating' },
    { key: 'market', label: 'Market Value' },
    { key: 'fifa', label: 'FIFA Ranking' },
  ];

  return (
    <div className="space-y-3">
      <label className="block text-sm font-medium text-gray-700">Composite Weights</label>
      {weights.map(({ key, label }) => (
        <div key={key} className="space-y-1">
          <div className="flex justify-between text-sm">
            <span className="text-gray-600">{label}</span>
            <span className="font-medium text-gray-900">{(value[key] * 100).toFixed(0)}%</span>
          </div>
          <input
            type="range"
            min="0"
            max="100"
            value={value[key] * 100}
            onChange={(e) => handleChange(key, parseInt(e.target.value) / 100)}
            className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-blue-600"
          />
        </div>
      ))}
    </div>
  );
}
