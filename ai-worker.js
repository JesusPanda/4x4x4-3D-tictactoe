import init, { get_best_move } from './tic-tac-toe-ai/pkg/tic_tac_toe_ai.js';

let isInitialized = false;

self.onmessage = async function (e) {
    const { type, p1Mask, p2Mask, aiIsPlayer1, timeLimit } = e.data;

    if (type === 'stop') {
        // Since Wasm is synchronous, we can't interrupt mid-computation easily.
        // We just ignore the next result if we could, but here we just return.
        // (Real cancellation requires SharedArrayBuffer flags or polling time in Wasm, which we do)
        return;
    }

    if (!isInitialized) {
        await init();
        isInitialized = true;
    }

    // p1Mask, p2Mask are strings or BigInts
    const p1 = BigInt(p1Mask);
    const p2 = BigInt(p2Mask);

    try {
        const move = get_best_move(p1, p2, aiIsPlayer1, timeLimit);
        // move is {x, y, z, score}
        self.postMessage({ type: 'result', move });
    } catch (err) {
        console.error("Wasm AI Error:", err);
    }
};
