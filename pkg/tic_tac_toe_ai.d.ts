/* tslint:disable */
/* eslint-disable */

export class Move {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    score: number;
    x: number;
    y: number;
    z: number;
}

export class SearchResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    depth: number;
    score: number;
    x: number;
    y: number;
    z: number;
}

export function clear_tt(): void;

/**
 * Main entry point - returns result with depth info for progress display
 */
export function get_best_move(p1_mask: bigint, p2_mask: bigint, ai_is_p1: boolean, time_limit_ms: number, progress_callback: Function): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_get_move_score: (a: number) => number;
    readonly __wbg_get_move_x: (a: number) => number;
    readonly __wbg_get_move_y: (a: number) => number;
    readonly __wbg_get_move_z: (a: number) => number;
    readonly __wbg_get_searchresult_depth: (a: number) => number;
    readonly __wbg_move_free: (a: number, b: number) => void;
    readonly __wbg_searchresult_free: (a: number, b: number) => void;
    readonly __wbg_set_move_score: (a: number, b: number) => void;
    readonly __wbg_set_move_x: (a: number, b: number) => void;
    readonly __wbg_set_move_y: (a: number, b: number) => void;
    readonly __wbg_set_move_z: (a: number, b: number) => void;
    readonly __wbg_set_searchresult_depth: (a: number, b: number) => void;
    readonly get_best_move: (a: bigint, b: bigint, c: number, d: number, e: any) => any;
    readonly __wbg_set_searchresult_score: (a: number, b: number) => void;
    readonly clear_tt: () => void;
    readonly __wbg_get_searchresult_x: (a: number) => number;
    readonly __wbg_get_searchresult_y: (a: number) => number;
    readonly __wbg_get_searchresult_z: (a: number) => number;
    readonly __wbg_get_searchresult_score: (a: number) => number;
    readonly __wbg_set_searchresult_x: (a: number, b: number) => void;
    readonly __wbg_set_searchresult_y: (a: number, b: number) => void;
    readonly __wbg_set_searchresult_z: (a: number, b: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
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
