import Link from 'next/link';

export default function Home() {
  return (
    <main className="min-h-screen bg-slate-900 text-white flex flex-col items-center justify-center gap-8 p-8">
      <h1 className="text-4xl font-bold">Fractal Puzzle Generator</h1>
      <p className="text-slate-400 text-center max-w-xl">
        Design laser-cut fractal jigsaw puzzles. Preview in 2D or 3D, export
        SVG files ready for your laser cutter.
      </p>
      <Link
        href="/configurator"
        className="px-6 py-3 bg-blue-600 hover:bg-blue-500 rounded-lg font-semibold transition-colors"
      >
        Open Configurator
      </Link>
    </main>
  );
}
