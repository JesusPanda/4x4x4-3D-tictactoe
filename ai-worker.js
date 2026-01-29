import init, { search_depth, clear_tt } from './tic-tac-toe-ai/pkg/tic_tac_toe_ai.js';

let isInitialized = false;

self.onmessage = async function (e) {
    const { type, p1Mask, p2Mask, aiIsPlayer1, timeLimit, clearCache } = e.data;

    if (type === 'stop') {
        // worker termination is handled by main thread, but we can set a flag if needed
        return;
    }

    if (type === 'clear_cache') {
        if (isInitialized) clear_tt();
        return;
    }

    if (!isInitialized) {
        await init();
        isInitialized = true;
    }

    if (clearCache) clear_tt();

    const p1 = BigInt(p1Mask);
    const p2 = BigInt(p2Mask);

    // Calculate stop time in JS (performance.now is relative, use Date.now for sync with Wasm?)
    // Wasm uses js_sys::Date::now(), which is Date.now().
    // So we should pass the absolute stop time timestamp.
    const startTime = Date.now();
    const stopTime = startTime + timeLimit; // timeLimit is in ms

    let bestMoveSoFar = null;

    try {
        // Iterative Deepening Loop in JS
        for (let depth = 1; depth <= 20; depth++) {

            // Call Wasm for ONE depth
            const result = search_depth(p1, p2, aiIsPlayer1, depth, stopTime);

            // result is {x, y, z, score, depth, time_abort}
            // or null (if no moves)

            if (!result) {
                // No moves possible?
                if (depth === 1) bestMoveSoFar = null;
                break;
            }

            if (result.time_abort) {
                // Search aborted due to time
                // If we have a previous best move, use it.
                // If this was depth 1 and we timed out, 'result' might be garbage or valid?
                // search_depth returns "best found so far" even on abort usually, but let's be safe.
                // If we found a move at this depth before abort, getting it is good.
                // But my Wasm code returns the accumulated best if !time_abort. 
                // Wait, I updated Wasm to return move even on abort? No, check line 265 in lib.rs
                // It returns result ONLY if `!time_abort`. 
                // If `time_abort` is true, it returns `serialize_result(..., true)` which might have None move.

                // If result.x is valid (not 0/0/0 placeholder unless that's valid), use it?
                // Actually my Wasm returns `SearchResult` with `time_abort: true` and dummy data if aborted at root.
                // So we should just stop and use `bestMoveSoFar`.
                console.log(`Aborted at depth ${depth}`);
                break;
            } else {
                // Completed depth successfully
                bestMoveSoFar = result;

                // Send progress update
                self.postMessage({
                    type: 'progress',
                    depth: result.depth,
                    score: result.score
                });

                // Check for forced win
                if (result.score >= 900000) {
                    break;
                }
            }

            // Double check time in JS loop just to be sure
            if (Date.now() >= stopTime) {
                break;
            }
        }

        self.postMessage({
            type: 'result',
            move: bestMoveSoFar
        });

    } catch (err) {
        console.error("Wasm AI Error:", err);
        self.postMessage({ type: 'error', message: err.toString() });
    }
};
