import init, { get_best_move, clear_tt } from './tic-tac-toe-ai/pkg/tic_tac_toe_ai.js';

let isInitialized = false;

self.onmessage = async function (e) {
    const { type, p1Mask, p2Mask, aiIsPlayer1, timeLimit, clearCache } = e.data;

    if (type === 'stop') {
        // Wasm respects time limits internally, so "stop" is handled by the time check
        return;
    }

    if (type === 'clear_cache') {
        if (isInitialized) {
            clear_tt();
        }
        return;
    }

    if (!isInitialized) {
        await init();
        isInitialized = true;
    }

    // Clear TT on new game if requested
    if (clearCache) {
        clear_tt();
    }

    // p1Mask, p2Mask are strings or BigInts
    const p1 = BigInt(p1Mask);
    const p2 = BigInt(p2Mask);

    // Progress callback - called by Wasm after each depth completes
    const progressCallback = (depth, score) => {
        self.postMessage({ type: 'progress', depth, score });
    };

    try {
        const result = get_best_move(p1, p2, aiIsPlayer1, timeLimit, progressCallback);
        // result is {x, y, z, score, depth}
        if (result) {
            self.postMessage({
                type: 'result',
                move: {
                    x: result.x,
                    y: result.y,
                    z: result.z,
                    score: result.score,
                    depth: result.depth
                }
            });
        } else {
            self.postMessage({ type: 'result', move: null });
        }
    } catch (err) {
        console.error("Wasm AI Error:", err);
        self.postMessage({ type: 'error', message: err.toString() });
    }
};
