/* tslint:disable */
/* eslint-disable */

export class SearchResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    depth: number;
    score: number;
    time_abort: boolean;
    x: number;
    y: number;
    z: number;
}

export function clear_tt(): void;

/**
 * Runs ONE depth of the search.
 * Returns SearchResult. If time_abort is true, score is invalid.
 */
export function search_depth(p1_mask: bigint, p2_mask: bigint, ai_is_p1: boolean, depth: number, stop_time: number): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_get_searchresult_depth: (a: number) => number;
    readonly __wbg_get_searchresult_score: (a: number) => number;
    readonly __wbg_get_searchresult_time_abort: (a: number) => number;
    readonly __wbg_get_searchresult_x: (a: number) => number;
    readonly __wbg_get_searchresult_y: (a: number) => number;
    readonly __wbg_get_searchresult_z: (a: number) => number;
    readonly __wbg_searchresult_free: (a: number, b: number) => void;
    readonly __wbg_set_searchresult_depth: (a: number, b: number) => void;
    readonly __wbg_set_searchresult_score: (a: number, b: number) => void;
    readonly __wbg_set_searchresult_time_abort: (a: number, b: number) => void;
    readonly __wbg_set_searchresult_x: (a: number, b: number) => void;
    readonly __wbg_set_searchresult_y: (a: number, b: number) => void;
    readonly __wbg_set_searchresult_z: (a: number, b: number) => void;
    readonly search_depth: (a: bigint, b: bigint, c: number, d: number, e: number) => any;
    readonly clear_tt: () => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
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
