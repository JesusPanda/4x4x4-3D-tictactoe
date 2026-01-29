/**
 * AI Worker for 4x4x4 Tic-Tac-Toe
 * Optimized with Iterative Deepening, Time Limits, and Alpha-Beta pruning
 */

// Store best move found so far (for "Play Now" interruption)
let currentBestMove = null;
let searchStartTime = 0;
let timeLimit = 3000;
let shouldStop = false;

// Precompute all 76 winning lines as BigInt bitmasks
function cellToBit(x, y, z) {
    return BigInt(1) << BigInt(y * 16 + z * 4 + x);
}

function generateWinningLines() {
    const lines = [];

    // 1. Rows (along X) - 16 lines
    for (let y = 0; y < 4; y++) {
        for (let z = 0; z < 4; z++) {
            let mask = 0n;
            for (let x = 0; x < 4; x++) mask |= cellToBit(x, y, z);
            lines.push(mask);
        }
    }

    // 2. Columns (along Z) - 16 lines
    for (let y = 0; y < 4; y++) {
        for (let x = 0; x < 4; x++) {
            let mask = 0n;
            for (let z = 0; z < 4; z++) mask |= cellToBit(x, y, z);
            lines.push(mask);
        }
    }

    // 3. Pillars (along Y) - 16 lines
    for (let x = 0; x < 4; x++) {
        for (let z = 0; z < 4; z++) {
            let mask = 0n;
            for (let y = 0; y < 4; y++) mask |= cellToBit(x, y, z);
            lines.push(mask);
        }
    }

    // 4. 2D Diagonals on XZ planes (fixed Y) - 8 lines
    for (let y = 0; y < 4; y++) {
        let mask1 = 0n, mask2 = 0n;
        for (let i = 0; i < 4; i++) {
            mask1 |= cellToBit(i, y, i);
            mask2 |= cellToBit(3 - i, y, i);
        }
        lines.push(mask1, mask2);
    }

    // 5. 2D Diagonals on XY planes (fixed Z) - 8 lines
    for (let z = 0; z < 4; z++) {
        let mask1 = 0n, mask2 = 0n;
        for (let i = 0; i < 4; i++) {
            mask1 |= cellToBit(i, i, z);
            mask2 |= cellToBit(i, 3 - i, z);
        }
        lines.push(mask1, mask2);
    }

    // 6. 2D Diagonals on YZ planes (fixed X) - 8 lines
    for (let x = 0; x < 4; x++) {
        let mask1 = 0n, mask2 = 0n;
        for (let i = 0; i < 4; i++) {
            mask1 |= cellToBit(x, i, i);
            mask2 |= cellToBit(x, i, 3 - i);
        }
        lines.push(mask1, mask2);
    }

    // 7. 3D Main Diagonals - 4 lines
    let diag1 = 0n, diag2 = 0n, diag3 = 0n, diag4 = 0n;
    for (let i = 0; i < 4; i++) {
        diag1 |= cellToBit(i, i, i);
        diag2 |= cellToBit(3 - i, i, i);
        diag3 |= cellToBit(i, i, 3 - i);
        diag4 |= cellToBit(3 - i, i, 3 - i);
    }
    lines.push(diag1, diag2, diag3, diag4);

    return lines;
}

const WINNING_LINES = generateWinningLines();

// Move ordering: center cells first (improves pruning)
function generateMoveOrder() {
    const scored = [];
    for (let i = 0; i < 64; i++) {
        const x = i % 4;
        const z = Math.floor(i / 4) % 4;
        const y = Math.floor(i / 16);
        const dx = Math.abs(x - 1.5);
        const dy = Math.abs(y - 1.5);
        const dz = Math.abs(z - 1.5);
        scored.push({ index: i, dist: dx + dy + dz });
    }
    scored.sort((a, b) => a.dist - b.dist);
    return scored.map(s => s.index);
}

const MOVE_ORDER = generateMoveOrder();

// Fast popcount using lookup
function popcount(n) {
    let count = 0;
    while (n > 0n) {
        count += Number(n & 1n);
        n >>= 1n;
    }
    return count;
}

// Check if time limit exceeded
function isTimeUp() {
    return shouldStop || (performance.now() - searchStartTime > timeLimit);
}

// Evaluate board state heuristically
function evaluate(myMask, oppMask) {
    let score = 0;

    for (const line of WINNING_LINES) {
        const myCount = popcount(myMask & line);
        const oppCount = popcount(oppMask & line);

        if (myCount === 4) return 1000000;
        if (oppCount === 4) return -1000000;

        if (oppCount === 0 && myCount > 0) {
            if (myCount === 3) score += 10000;
            else if (myCount === 2) score += 100;
            else score += 1;
        }

        if (myCount === 0 && oppCount > 0) {
            if (oppCount === 3) score -= 10000;
            else if (oppCount === 2) score -= 100;
            else score -= 1;
        }
    }

    return score;
}

// Check if game is won
function checkWin(mask) {
    for (const line of WINNING_LINES) {
        if ((mask & line) === line) return true;
    }
    return false;
}

// Get available moves
function getAvailableMoves(p1Mask, p2Mask) {
    const occupied = p1Mask | p2Mask;
    const moves = [];
    for (const idx of MOVE_ORDER) {
        const bit = BigInt(1) << BigInt(idx);
        if ((occupied & bit) === 0n) moves.push(idx);
    }
    return moves;
}

// Minimax with Alpha-Beta pruning and time check
function minimax(p1Mask, p2Mask, depth, alpha, beta, isMaximizing, aiIsPlayer1) {
    // Check time limit periodically
    if (depth > 0 && isTimeUp()) return 0;

    const myMask = aiIsPlayer1 ? p1Mask : p2Mask;
    const oppMask = aiIsPlayer1 ? p2Mask : p1Mask;

    if (checkWin(myMask)) return 1000000 + depth;
    if (checkWin(oppMask)) return -1000000 - depth;

    if (depth === 0) return evaluate(myMask, oppMask);

    const moves = getAvailableMoves(p1Mask, p2Mask);
    if (moves.length === 0) return 0;

    if (isMaximizing) {
        let maxEval = -Infinity;
        for (const moveIdx of moves) {
            if (isTimeUp()) break;
            const bit = BigInt(1) << BigInt(moveIdx);
            const newP1 = aiIsPlayer1 ? (p1Mask | bit) : p1Mask;
            const newP2 = aiIsPlayer1 ? p2Mask : (p2Mask | bit);
            const evalScore = minimax(newP1, newP2, depth - 1, alpha, beta, false, aiIsPlayer1);
            maxEval = Math.max(maxEval, evalScore);
            alpha = Math.max(alpha, evalScore);
            if (beta <= alpha) break;
        }
        return maxEval;
    } else {
        let minEval = Infinity;
        for (const moveIdx of moves) {
            if (isTimeUp()) break;
            const bit = BigInt(1) << BigInt(moveIdx);
            const newP1 = aiIsPlayer1 ? p1Mask : (p1Mask | bit);
            const newP2 = aiIsPlayer1 ? (p2Mask | bit) : p2Mask;
            const evalScore = minimax(newP1, newP2, depth - 1, alpha, beta, true, aiIsPlayer1);
            minEval = Math.min(minEval, evalScore);
            beta = Math.min(beta, evalScore);
            if (beta <= alpha) break;
        }
        return minEval;
    }
}

// Find best move at a specific depth
function searchAtDepth(p1Mask, p2Mask, aiIsPlayer1, depth) {
    const moves = getAvailableMoves(p1Mask, p2Mask);
    if (moves.length === 0) return null;

    let bestMove = moves[0];
    let bestScore = -Infinity;

    for (const moveIdx of moves) {
        if (isTimeUp()) break;

        const bit = BigInt(1) << BigInt(moveIdx);
        const newP1 = aiIsPlayer1 ? (p1Mask | bit) : p1Mask;
        const newP2 = aiIsPlayer1 ? p2Mask : (p2Mask | bit);

        const score = minimax(newP1, newP2, depth - 1, -Infinity, Infinity, false, aiIsPlayer1);

        if (score > bestScore) {
            bestScore = score;
            bestMove = moveIdx;
        }

        if (score >= 1000000) break; // Winning move found
    }

    const x = bestMove % 4;
    const z = Math.floor(bestMove / 4) % 4;
    const y = Math.floor(bestMove / 16);

    return { x, y, z, score: bestScore };
}

// Iterative deepening search
function iterativeDeepening(p1Mask, p2Mask, aiIsPlayer1) {
    currentBestMove = null;
    let depth = 1;
    const maxDepth = 10; // Upper bound

    // Start with a random move from center-prioritized list
    const moves = getAvailableMoves(p1Mask, p2Mask);
    if (moves.length > 0) {
        const idx = moves[0];
        currentBestMove = {
            x: idx % 4,
            z: Math.floor(idx / 4) % 4,
            y: Math.floor(idx / 16),
            score: 0,
            depth: 0
        };
    }

    while (depth <= maxDepth && !isTimeUp()) {
        const result = searchAtDepth(p1Mask, p2Mask, aiIsPlayer1, depth);

        if (result && !isTimeUp()) {
            currentBestMove = { ...result, depth };

            // Report progress
            self.postMessage({
                type: 'progress',
                depth,
                score: result.score
            });

            // If winning move found, stop searching
            if (result.score >= 1000000) break;
        }

        depth++;
    }

    return currentBestMove;
}

// Handle messages
self.onmessage = function (e) {
    const { type, p1Mask, p2Mask, aiIsPlayer1, timeLimit: msgTimeLimit } = e.data;

    // Handle "play now" interrupt
    if (type === 'stop') {
        shouldStop = true;
        if (currentBestMove) {
            self.postMessage({ type: 'result', move: currentBestMove });
        }
        return;
    }

    // Start new search
    shouldStop = false;
    searchStartTime = performance.now();
    timeLimit = msgTimeLimit || 3000; // Default 3 seconds

    const p1 = BigInt(p1Mask);
    const p2 = BigInt(p2Mask);

    const result = iterativeDeepening(p1, p2, aiIsPlayer1);

    if (!shouldStop) {
        self.postMessage({ type: 'result', move: result });
    }
};
