'use client';

import dynamic from 'next/dynamic';

// SSR=false prevents WebGL errors during Node-side rendering
const PuzzleScene = dynamic(
  () => import('./PuzzleScene').then((m) => m.PuzzleScene),
  { ssr: false, loading: () => <div className="w-full h-96 bg-slate-900 rounded animate-pulse" /> }
);

interface Props {
  svgString: string;
}

export function PuzzlePreview3D({ svgString }: Props) {
  return <PuzzleScene svgString={svgString} />;
}
