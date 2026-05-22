'use client';

import { useRef } from 'react';

interface Props {
  onLoad: (file: File, scale: number) => void;
  onClear: () => void;
  hasBorder: boolean;
  error: string | null;
}

export function BorderUpload({ onLoad, onClear, hasBorder, error }: Props) {
  const fileRef = useRef<HTMLInputElement>(null);
  const scaleRef = useRef<HTMLInputElement>(null);

  const handleLoad = () => {
    const file = fileRef.current?.files?.[0];
    if (!file) return;
    const scale = parseFloat(scaleRef.current?.value || '1') || 1;
    onLoad(file, scale);
  };

  return (
    <div className="flex flex-wrap items-center gap-2">
      <input
        ref={fileRef}
        type="file"
        accept=".svg"
        className="text-sm text-slate-300 file:mr-2 file:py-1 file:px-2 file:rounded file:border-0 file:bg-slate-600 file:text-white file:cursor-pointer"
      />
      <label className="text-sm text-slate-400">
        Scale:
        <input
          ref={scaleRef}
          type="number"
          defaultValue="1"
          step="0.1"
          min="0.1"
          className="ml-1 w-16 text-sm bg-slate-700 border border-slate-600 rounded px-1 py-0.5 text-white"
        />
      </label>
      <button
        onClick={handleLoad}
        className="px-3 py-1.5 text-sm bg-blue-600 hover:bg-blue-500 text-white rounded"
      >
        Load border
      </button>
      {hasBorder && (
        <button
          onClick={onClear}
          className="px-3 py-1.5 text-sm bg-slate-600 hover:bg-slate-500 text-white rounded"
        >
          Clear
        </button>
      )}
      {error && <span className="text-red-400 text-xs">{error}</span>}
    </div>
  );
}
