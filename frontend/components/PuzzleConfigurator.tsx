'use client';

import { useState, useCallback } from 'react';
import { usePuzzleWasm } from '@/hooks/usePuzzleWasm';
import { usePuzzleGenerator } from '@/hooks/usePuzzleGenerator';
import { useBorderMask } from '@/hooks/useBorderMask';
import { PuzzlePreview2D } from './PuzzlePreview2D';
import { PuzzlePreview3D } from './PuzzlePreview3D';
import { ExportButtons } from './ExportButtons';
import { BorderUpload } from './BorderUpload';
import type { PuzzleParams } from '@/hooks/usePuzzleGenerator';

const DEFAULTS: PuzzleParams = {
  seed: 42,
  ncols: 10,
  nrows: 10,
  minPiece: 4,
  maxPiece: 50,
  tileRadius: 6,
  frame: 6,
  frameCorner: 4,
  arcShape: 0,
};

type Preview = '2d' | '3d';

function SliderField({
  label, name, value, min, max, step, unit, onChange,
}: {
  label: string; name: keyof PuzzleParams; value: number;
  min: number; max: number; step: number; unit?: string;
  onChange: (key: keyof PuzzleParams, val: number) => void;
}) {
  return (
    <div className="grid grid-cols-[10rem_5rem_1fr] items-center gap-2">
      <label className="text-sm text-slate-300">{label}</label>
      <input
        type="number"
        value={value}
        min={min}
        max={max}
        step={step}
        onChange={(e) => onChange(name, parseFloat(e.target.value))}
        className="w-full bg-slate-700 border border-slate-600 rounded px-2 py-1 text-sm text-white"
      />
      <input
        type="range"
        value={value}
        min={min}
        max={max}
        step={step}
        onChange={(e) => onChange(name, parseFloat(e.target.value))}
        className="w-full accent-blue-500"
      />
      {unit && <span className="text-xs text-slate-500 col-start-2">{unit}</span>}
    </div>
  );
}

export function PuzzleConfigurator() {
  const [params, setParams] = useState<PuzzleParams>(DEFAULTS);
  const [preview, setPreview] = useState<Preview>('2d');

  const { ready, module } = usePuzzleWasm();
  const { result, generating, error, generate, exportSvg } = usePuzzleGenerator(module);
  const { borderMask, borderError, loadBorder, clearBorder } = useBorderMask(params.tileRadius);

  const setParam = useCallback((key: keyof PuzzleParams, val: number) => {
    setParams((p) => ({ ...p, [key]: val }));
  }, []);

  const handleGenerate = () => generate(params, borderMask);

  const handleExport = useCallback(
    (mode: number) => {
      const svg = exportSvg(params, borderMask, mode);
      if (!svg) return;
      const blob = new Blob([svg], { type: 'image/svg+xml' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `jigsaw-mode${mode}.svg`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    },
    [params, borderMask, exportSvg]
  );

  const widthMm = params.ncols * 2 * params.tileRadius + 2 * params.frame;
  const heightMm = params.nrows * 2 * params.tileRadius + 2 * params.frame;

  return (
    <div className="flex flex-col lg:flex-row gap-6 p-6 min-h-screen bg-slate-900 text-white">
      {/* Controls */}
      <div className="lg:w-80 shrink-0 space-y-4">
        <h1 className="text-xl font-semibold">Fractal Puzzle Generator</h1>

        {!ready && (
          <div className="text-yellow-400 text-sm">Loading Wasm engine…</div>
        )}

        <div className="space-y-3">
          <SliderField label="Seed" name="seed" value={params.seed} min={0} max={9999} step={1} onChange={setParam} />
          <SliderField label="Columns" name="ncols" value={params.ncols} min={2} max={100} step={1} onChange={setParam} />
          <SliderField label="Rows" name="nrows" value={params.nrows} min={2} max={100} step={1} onChange={setParam} />
          <SliderField label="Tile radius" name="tileRadius" value={params.tileRadius} min={2} max={20} step={0.5} unit="mm" onChange={setParam} />
          <SliderField label="Frame" name="frame" value={params.frame} min={0} max={20} step={0.5} unit="mm" onChange={setParam} />
          <SliderField label="Frame corner" name="frameCorner" value={params.frameCorner} min={0} max={10} step={0.5} unit="mm" onChange={setParam} />
          <SliderField label="Min piece" name="minPiece" value={params.minPiece} min={1} max={20} step={1} unit="tiles" onChange={setParam} />
          <SliderField label="Max piece" name="maxPiece" value={params.maxPiece} min={1} max={200} step={1} unit="tiles" onChange={setParam} />
        </div>

        <div className="space-y-1">
          <p className="text-sm text-slate-400">Tile shape</p>
          {(['Circular', 'Square', 'Octagonal'] as const).map((label, i) => (
            <label key={i} className="flex items-center gap-2 text-sm">
              <input
                type="radio"
                name="arcShape"
                checked={params.arcShape === i}
                onChange={() => setParam('arcShape', i)}
                className="accent-blue-500"
              />
              {label}
            </label>
          ))}
        </div>

        <div className="space-y-2">
          <p className="text-sm text-slate-400">Custom border (SVG)</p>
          <BorderUpload
            onLoad={loadBorder}
            onClear={clearBorder}
            hasBorder={!!borderMask}
            error={borderError}
          />
        </div>

        <p className="text-xs text-slate-500">
          Size: {widthMm.toFixed(0)} × {heightMm.toFixed(0)} mm
        </p>

        <button
          onClick={handleGenerate}
          disabled={!ready || generating}
          className="w-full py-2 bg-blue-600 hover:bg-blue-500 disabled:opacity-40 disabled:cursor-not-allowed rounded font-medium transition-colors"
        >
          {generating ? 'Generating…' : 'Generate Jigsaw'}
        </button>

        {result && (
          <p className="text-sm text-slate-300">
            {result.pieceCount} pieces · {result.widthMm.toFixed(0)}×{result.heightMm.toFixed(0)} mm
          </p>
        )}

        {error && <p className="text-red-400 text-sm">{error}</p>}

        {result && (
          <div className="space-y-2">
            <p className="text-sm text-slate-400">Download SVG</p>
            <ExportButtons onExport={handleExport} disabled={!ready || generating} />
          </div>
        )}
      </div>

      {/* Preview */}
      <div className="flex-1 space-y-3">
        <div className="flex gap-2">
          <button
            onClick={() => setPreview('2d')}
            className={`px-3 py-1 rounded text-sm ${preview === '2d' ? 'bg-blue-600' : 'bg-slate-700 hover:bg-slate-600'}`}
          >
            2D
          </button>
          <button
            onClick={() => setPreview('3d')}
            className={`px-3 py-1 rounded text-sm ${preview === '3d' ? 'bg-blue-600' : 'bg-slate-700 hover:bg-slate-600'}`}
          >
            3D
          </button>
        </div>

        {result ? (
          preview === '2d' ? (
            <PuzzlePreview2D svgString={result.svgString} />
          ) : (
            <PuzzlePreview3D svgString={result.svgString} />
          )
        ) : (
          <div className="w-full h-96 bg-slate-800 rounded flex items-center justify-center text-slate-500 text-sm">
            Press Generate to create a puzzle
          </div>
        )}
      </div>
    </div>
  );
}
