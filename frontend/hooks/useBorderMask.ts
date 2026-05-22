'use client';

import { useCallback, useState } from 'react';
import { plotpath } from '@/lib/plotpath';

// flatten is a CommonJS library, imported via require
const flatten = typeof window !== 'undefined'
  ? require('@/lib/flatten.js')
  : null;

export interface BorderMask {
  mask: Uint8Array;
  rows: number;
  cols: number;
  widthMm: number;
  heightMm: number;
}

export function useBorderMask(tileRadius: number) {
  const [borderMask, setBorderMask] = useState<BorderMask | null>(null);
  const [borderError, setBorderError] = useState<string | null>(null);

  const loadBorder = useCallback((file: File, scaleFactor: number) => {
    setBorderError(null);
    const reader = new FileReader();
    reader.readAsText(file);
    reader.onload = () => {
      try {
        const parser = new DOMParser();
        const xmlDoc = parser.parseFromString(reader.result as string, 'image/svg+xml');
        const svgEl = xmlDoc.getElementsByTagName('svg')[0];
        if (!svgEl) {
          setBorderError('Invalid SVG file');
          return;
        }

        // Attach to DOM temporarily for CTM/flatten operations
        svgEl.style.visibility = 'hidden';
        svgEl.style.position = 'absolute';
        document.body.appendChild(svgEl);

        // Flatten transforms (same flow as index.html:879-918)
        flatten(svgEl, false, true);

        const pathsInSvg = svgEl.getElementsByTagName('path');
        const pgroup = document.createElementNS(svgEl.namespaceURI, 'g');
        const vt = (pathsInSvg[0] as SVGPathElement).getCTM()!;
        svgEl.appendChild(pgroup);

        for (const p of Array.from(pathsInSvg)) {
          const np = document.createElementNS(svgEl.namespaceURI, 'path') as SVGPathElement;
          np.setAttribute('d', p.getAttribute('d') || '');
          pgroup.appendChild(np);
        }

        // De-viewboxify → scale → translate to origin
        pgroup.setAttribute('transform', `matrix(${vt.a},${vt.b},${vt.c},${vt.d},${vt.e},${vt.f})`);
        flatten(svgEl, false, true);
        pgroup.setAttribute('transform', `scale(${scaleFactor})`);
        flatten(svgEl, false, true);
        const bbox0 = svgEl.getBBox();
        pgroup.setAttribute('transform', `translate(${-bbox0.x},${-bbox0.y})`);
        flatten(svgEl, false, true);

        const bbox = svgEl.getBBox();
        const widthMm = bbox.width + bbox.x;
        const heightMm = bbox.height + bbox.y;

        const dia = 2 * tileRadius;
        const ncols = Math.ceil(widthMm / dia) + 1;
        const nrows = Math.ceil(heightMm / dia) + 1;

        // Rasterize all paths via plotpath()
        const maskgrid = new Uint8Array(nrows * ncols);
        const allPts: Array<{ x: number; y: number; rx: number; ry: number }> = [];

        for (const p of Array.from(pgroup.getElementsByTagName('path'))) {
          const pts = plotpath(p as SVGPathElement, dia);
          for (const pt of pts) {
            allPts.push(pt);
            if (pt.y >= 0 && pt.x >= 0 && pt.y < nrows && pt.x < ncols) {
              maskgrid[pt.y * ncols + pt.x] = 1;
            }
          }
        }

        // Flood-fill exterior (simplified — marks boundary cells)
        // The full flood-fill happens inside Rust set_mask(); here we just pass the raw raster
        const mask = new Uint8Array(nrows * ncols);
        mask.set(maskgrid);

        document.body.removeChild(svgEl);
        setBorderMask({ mask, rows: nrows, cols: ncols, widthMm, heightMm });
      } catch (e) {
        setBorderError(String(e));
      }
    };
  }, [tileRadius]);

  const clearBorder = useCallback(() => setBorderMask(null), []);

  return { borderMask, borderError, loadBorder, clearBorder };
}
