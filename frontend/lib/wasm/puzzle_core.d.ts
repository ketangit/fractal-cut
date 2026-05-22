/* tslint:disable */
/* eslint-disable */

export class PuzzleHandle {
    free(): void;
    [Symbol.dispose](): void;
    coloring_seed(): number;
    export_svg(mode: number): string;
    fill_holes(): boolean;
    fill_holes_partial(): boolean;
    generate(): void;
    constructor(config_json: string);
    piece_count(): number;
    /**
     * Receive pre-rasterized mask from JS plotpath().
     */
    set_mask(mask: Uint8Array, rows: number, cols: number): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_puzzlehandle_free: (a: number, b: number) => void;
    readonly puzzlehandle_coloring_seed: (a: number) => number;
    readonly puzzlehandle_export_svg: (a: number, b: number) => [number, number];
    readonly puzzlehandle_fill_holes: (a: number) => number;
    readonly puzzlehandle_fill_holes_partial: (a: number) => number;
    readonly puzzlehandle_generate: (a: number) => void;
    readonly puzzlehandle_new: (a: number, b: number) => [number, number, number];
    readonly puzzlehandle_piece_count: (a: number) => number;
    readonly puzzlehandle_set_mask: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
