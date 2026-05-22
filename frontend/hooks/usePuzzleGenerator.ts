'use client';

import { useCallback, useState } from 'react';
import type { RefObject } from 'react';
import type { PuzzleHandle } from '@/lib/wasm/puzzle_core';
import type { BorderMask } from './useBorderMask';

interface WasmModule { PuzzleHandle: { new(config: string): PuzzleHandle } }

export interface PuzzleParams {
  seed: number;
  ncols: number;
  nrows: number;
  minPiece: number;
  maxPiece: number;
  tileRadius: number;
  frame: number;
  frameCorner: number;
  arcShape: number;
}

export interface PuzzleResult {
  svgString: string;
  pieceCount: number;
  widthMm: number;
  heightMm: number;
}

export function usePuzzleGenerator(moduleRef: RefObject<WasmModule | null>) {
  const [result, setResult] = useState<PuzzleResult | null>(null);
  const [generating, setGenerating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const generate = useCallback(
    (params: PuzzleParams, borderMask: BorderMask | null) => {
      const mod = moduleRef.current;
      if (!mod) {
        setError('Wasm module not ready');
        return;
      }
      setGenerating(true);
      setError(null);

      // Defer to next tick so React can update the loading state
      setTimeout(() => {
        try {
          const cfg = JSON.stringify({
            seed: params.seed,
            ncols: params.ncols,
            nrows: params.nrows,
            min_piece: params.minPiece,
            max_piece: params.maxPiece,
            tile_radius: params.tileRadius,
            frame: params.frame,
            frame_corner: params.frameCorner,
            arc_shape: params.arcShape,
          });

          const handle = new mod.PuzzleHandle(cfg);
          try {
            if (borderMask) {
              handle.set_mask(borderMask.mask, borderMask.rows, borderMask.cols);
            }
            handle.generate();
            while (handle.fill_holes()) {}
            handle.fill_holes_partial();

            const pieceCount: number = handle.piece_count();
            const svgString: string = handle.export_svg(3); // colored preview
            const widthMm = params.ncols * 2 * params.tileRadius + 2 * params.frame;
            const heightMm = params.nrows * 2 * params.tileRadius + 2 * params.frame;

            setResult({ svgString, pieceCount, widthMm, heightMm });
          } finally {
            handle.free();
          }
        } catch (e) {
          setError(String(e));
        }
        setGenerating(false);
      }, 0);
    },
    [moduleRef]
  );

  const exportSvg = useCallback(
    (params: PuzzleParams, borderMask: BorderMask | null, mode: number): string | null => {
      const mod = moduleRef.current;
      if (!mod) return null;
      try {
        const cfg = JSON.stringify({
          seed: params.seed,
          ncols: params.ncols,
          nrows: params.nrows,
          min_piece: params.minPiece,
          max_piece: params.maxPiece,
          tile_radius: params.tileRadius,
          frame: params.frame,
          frame_corner: params.frameCorner,
          arc_shape: params.arcShape,
        });
        const handle = new mod.PuzzleHandle(cfg);
        try {
          if (borderMask) {
            handle.set_mask(borderMask.mask, borderMask.rows, borderMask.cols);
          }
          handle.generate();
          while (handle.fill_holes()) {}
          handle.fill_holes_partial();
          return handle.export_svg(mode) as string;
        } finally {
          handle.free();
        }
      } catch (e) {
        return null;
      }
    },
    [moduleRef]
  );

  return { result, generating, error, generate, exportSvg };
}
