// Port of index.html:36-62 — adaptive arc-length sampling using browser DOM APIs.
// This is the ONLY place that uses getTotalLength/getPointAtLength.
// Output is passed to Wasm via PuzzleHandle.set_mask().

const UINC_START = 0.0078125;       // 1/128
const UINC_MIN = 0.0000152587890625; // 1/65536

interface GridPoint {
  x: number;
  y: number;
  rx: number; // real (unrounded) coordinates for frame expansion
  ry: number;
}

export function plotpath(path: SVGPathElement, scale: number): GridPoint[] {
  const len = path.getTotalLength();
  const cp: GridPoint[] = [];
  let uinc = UINC_START;

  for (let u = 0; u <= 1; u += uinc) {
    const raw = path.getPointAtLength(u * len);
    const p: GridPoint = {
      rx: raw.x,
      ry: raw.y,
      x: Math.round(raw.x / scale),
      y: Math.round(raw.y / scale),
    };
    if (cp.length) {
      const prev = cp[cp.length - 1];
      if (Math.abs(prev.x - p.x) > 1 || Math.abs(prev.y - p.y) > 1) {
        u -= uinc;
        uinc = uinc / 2;
        if (uinc < UINC_MIN) {
          uinc = UINC_START;
          cp.push(p);
        }
        continue;
      }
    }
    cp.push(p);
  }
  return cp;
}
