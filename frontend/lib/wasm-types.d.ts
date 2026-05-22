// Type declarations for the puzzle-core wasm-bindgen glue.
// The actual runtime module is loaded dynamically from /wasm/puzzle_core.js.

export interface PuzzleHandle {
  generate(): void;
  fill_holes(): boolean;
  fill_holes_partial(): boolean;
  set_mask(mask: Uint8Array, rows: number, cols: number): void;
  export_svg(mode: number): string;
  piece_count(): number;
  coloring_seed(): number;
  free(): void;
}

export interface PuzzleCoreModule {
  default(wasmUrl: string): Promise<void>;
  PuzzleHandle: {
    new(config_json: string): PuzzleHandle;
  };
}
