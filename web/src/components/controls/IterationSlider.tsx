import { formatNumber } from '../../utils/formatting';

interface IterationSliderProps {
  value: number;
  onChange: (iterations: number) => void;
}

const presets = [100, 1000, 10000, 100000];

// Use a logarithmic scale for the slider
const minLog = Math.log10(100);
const maxLog = Math.log10(100000);

function valueToSlider(value: number): number {
  return ((Math.log10(value) - minLog) / (maxLog - minLog)) * 100;
}

function sliderToValue(slider: number): number {
  const log = minLog + (slider / 100) * (maxLog - minLog);
  return Math.round(Math.pow(10, log));
}

export function IterationSlider({ value, onChange }: IterationSliderProps) {
  return (
    <div className="space-y-3">
      <div className="flex justify-between items-center">
        <label className="block text-sm font-medium text-gray-700">Iterations</label>
        <span className="text-sm font-medium text-blue-600">{formatNumber(value)}</span>
      </div>

      <input
        type="range"
        min="0"
        max="100"
        value={valueToSlider(value)}
        onChange={(e) => onChange(sliderToValue(parseInt(e.target.value)))}
        className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-blue-600"
      />

      <div className="flex justify-between gap-1">
        {presets.map((preset) => (
          <button
            key={preset}
            onClick={() => onChange(preset)}
            className={`flex-1 px-2 py-1 text-xs rounded transition-colors ${
              value === preset
                ? 'bg-blue-600 text-white'
                : 'bg-gray-100 text-gray-600 hover:bg-gray-200'
            }`}
          >
            {preset >= 1000 ? `${preset / 1000}K` : preset}
          </button>
        ))}
      </div>

      <p className="text-xs text-gray-500">
        More iterations = more accurate results, but slower computation.
      </p>
    </div>
  );
}
