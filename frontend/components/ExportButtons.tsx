'use client';

interface Props {
  onExport: (mode: number) => void;
  disabled: boolean;
}

const MODES = [
  { mode: 3, label: 'Colored SVG' },
  { mode: 0, label: 'Individual pieces (overlap)' },
  { mode: 1, label: 'Non-overlapping' },
  { mode: 2, label: 'Single path (Trotec)' },
];

export function ExportButtons({ onExport, disabled }: Props) {
  return (
    <div className="flex flex-wrap gap-2">
      {MODES.map(({ mode, label }) => (
        <button
          key={mode}
          onClick={() => onExport(mode)}
          disabled={disabled}
          className="px-3 py-1.5 text-sm bg-slate-700 hover:bg-slate-600 disabled:opacity-40 disabled:cursor-not-allowed text-white rounded transition-colors"
        >
          {label}
        </button>
      ))}
    </div>
  );
}
