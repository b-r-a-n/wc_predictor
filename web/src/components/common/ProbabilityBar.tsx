interface ProbabilityBarProps {
  value: number;
  color?: string;
  showLabel?: boolean;
  height?: 'sm' | 'md' | 'lg';
}

export function ProbabilityBar({
  value,
  color = 'bg-blue-500',
  showLabel = true,
  height = 'md',
}: ProbabilityBarProps) {
  const heightClasses = {
    sm: 'h-2',
    md: 'h-4',
    lg: 'h-6',
  };

  const percentage = Math.min(100, Math.max(0, value * 100));

  return (
    <div className="relative w-full">
      <div className={`${heightClasses[height]} bg-gray-200 rounded-full overflow-hidden`}>
        <div
          className={`${heightClasses[height]} ${color} transition-all duration-300`}
          style={{ width: `${percentage}%` }}
        />
      </div>
      {showLabel && (
        <span className="absolute inset-0 flex items-center justify-center text-xs font-medium text-gray-700">
          {percentage.toFixed(1)}%
        </span>
      )}
    </div>
  );
}
