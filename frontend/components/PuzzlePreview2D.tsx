'use client';

interface Props {
  svgString: string;
}

export function PuzzlePreview2D({ svgString }: Props) {
  return (
    <div
      className="w-full overflow-auto bg-white rounded border border-slate-200"
      dangerouslySetInnerHTML={{ __html: svgString }}
    />
  );
}
