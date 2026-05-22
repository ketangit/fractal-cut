'use client';

import { useEffect, useRef, useState } from 'react';
import type { PuzzleHandle } from '@/lib/wasm/puzzle_core';

export type { PuzzleHandle };

interface WasmModule {
  PuzzleHandle: { new(config: string): PuzzleHandle };
  default: () => Promise<void>;
}

let cached: WasmModule | null = null;
let initPromise: Promise<WasmModule> | null = null;

function loadWasm(): Promise<WasmModule> {
  if (cached) return Promise.resolve(cached);
  if (initPromise) return initPromise;

  initPromise = (async () => {
    // webpack asyncWebAssembly resolves puzzle_core_bg.wasm via import.meta.url
    const mod = await import('@/lib/wasm/puzzle_core') as unknown as WasmModule;
    await mod.default();
    cached = mod;
    return cached;
  })();

  return initPromise;
}

export function usePuzzleWasm() {
  const [ready, setReady] = useState(false);
  const moduleRef = useRef<WasmModule | null>(null);

  useEffect(() => {
    loadWasm().then((mod) => {
      moduleRef.current = mod;
      setReady(true);
    }).catch(console.error);
  }, []);

  return { ready, module: moduleRef };
}
