use wasm_bindgen::prelude::*;
use serde::Serialize;
use std::sync::OnceLock;
use std::collections::HashMap;
use std::cell::RefCell;

// --- Data Structures ---

#[wasm_bindgen]
#[derive(Clone, Copy, Serialize)]
pub struct SearchResult {
    pub x: u8,
    pub y: u8,
    pub z: u8,
    pub score: i32,
    pub depth: u8,
    pub time_abort: bool,
}

#[derive(Clone, Copy)]
struct TTEntry {
    depth: i8,
    score: i32,
    flag: u8,
    best_move: u8,
}

// --- Constants & Statics ---

const WIN_SCORE: i32 = 30_000; // Adjusted scale for tuple scoring
const INF: i32 = 50_000;
const TIME_CHECK_INTERVAL: u32 = 2047;

const TT_EXACT: u8 = 0;
const TT_LOWER: u8 = 1;
const TT_UPPER: u8 = 2;

// Patashnik's "Strategic Value" (Corners > Center > Edges)
// Adjusted for 0-63 index (y*16 + z*4 + x)
static POSITIONAL_WEIGHTS: [i32; 64] = [
    // y=0 (Outer)
    200, 10, 10, 200, 
    10,   5,  5,  10, 
    10,   5,  5,  10, 
    200, 10, 10, 200,
    // y=1 (Inner)
    10,   5,  5,  10, 
    5,   50, 50,   5, 
    5,   50, 50,   5, 
    10,   5,  5,  10,
    // y=2 (Inner)
    10,   5,  5,  10, 
    5,   50, 50,   5, 
    5,   50, 50,   5, 
    10,   5,  5,  10,
    // y=3 (Outer)
    200, 10, 10, 200, 
    10,   5,  5,  10, 
    10,   5,  5,  10, 
    200, 10, 10, 200,
];

static WINNING_LINES: OnceLock<[u64; 76]> = OnceLock::new();

// --- Thread Local Storage ---

thread_local! {
    static TT: RefCell<HashMap<u128, TTEntry>> = RefCell::new(HashMap::with_capacity(500_000));
    static NODE_COUNT: RefCell<u32> = const { RefCell::new(0) };
}

// --- Initialization ---

fn get_winning_lines() -> &'static [u64; 76] {
    WINNING_LINES.get_or_init(generate_winning_lines)
}

fn generate_winning_lines() -> [u64; 76] {
    let mut lines = [0u64; 76];
    let mut idx = 0;
    let cell = |x, y, z| 1u64 << (y * 16 + z * 4 + x);

    // Rows, Cols, Pillars
    for i in 0..4 {
        for j in 0..4 {
            // x-axis
            let mut m = 0; for k in 0..4 { m |= cell(k, i, j); } lines[idx] = m; idx += 1;
            // z-axis
            let mut m = 0; for k in 0..4 { m |= cell(j, i, k); } lines[idx] = m; idx += 1;
            // y-axis
            let mut m = 0; for k in 0..4 { m |= cell(j, k, i); } lines[idx] = m; idx += 1;
        }
    }

    // Face Diagonals
    for i in 0..4 {
        // xz
        let mut m1 = 0; let mut m2 = 0;
        for k in 0..4 { m1 |= cell(k, i, k); m2 |= cell(3-k, i, k); }
        lines[idx] = m1; idx += 1; lines[idx] = m2; idx += 1;
        // xy
        let mut m1 = 0; let mut m2 = 0;
        for k in 0..4 { m1 |= cell(k, k, i); m2 |= cell(3-k, k, i); }
        lines[idx] = m1; idx += 1; lines[idx] = m2; idx += 1;
        // yz
        let mut m1 = 0; let mut m2 = 0;
        for k in 0..4 { m1 |= cell(i, k, k); m2 |= cell(i, 3-k, k); }
        lines[idx] = m1; idx += 1; lines[idx] = m2; idx += 1;
    }

    // Space Diagonals
    let mut d1=0; let mut d2=0; let mut d3=0; let mut d4=0;
    for k in 0..4 {
        d1 |= cell(k, k, k);
        d2 |= cell(3-k, k, k);
        d3 |= cell(k, 3-k, k);
        d4 |= cell(3-k, 3-k, k);
    }
    lines[idx] = d1; idx += 1; lines[idx] = d2; idx += 1;
    lines[idx] = d3; idx += 1; lines[idx] = d4;

    lines
}

// --- Helper Functions ---

fn tt_key(p1: u64, p2: u64, is_p1: bool) -> u128 {
    let turn = if is_p1 { 1 } else { 0 };
    ((p1 as u128) << 65) | ((p2 as u128) << 1) | turn
}

#[wasm_bindgen]
pub fn clear_tt() {
    TT.with(|tt| tt.borrow_mut().clear());
}

fn get_forced_move(my_mask: u64, opp_mask: u64) -> Option<u8> {
    let lines = get_winning_lines();
    let occupied = my_mask | opp_mask;

    // 1. Win Now
    for &line in lines.iter() {
        if (my_mask & line).count_ones() == 3 && (opp_mask & line) == 0 {
            return Some((line & !occupied).trailing_zeros() as u8);
        }
    }
    // 2. Block Now
    for &line in lines.iter() {
        if (opp_mask & line).count_ones() == 3 && (my_mask & line) == 0 {
            return Some((line & !occupied).trailing_zeros() as u8);
        }
    }
    None
}

// --- Opening Book ---
fn get_book_move(p1: u64, p2: u64, is_p1: bool) -> Option<u8> {
    let count = (p1 | p2).count_ones();
    
    // Turn 1 (P1): Take a corner
    if count == 0 { return Some(0); }

    // Turn 2 (P2): If P1 took corner, P2 MUST take center or adjacent corner
    // Patashnik's strategy dictates taking the center core to stop corner forks.
    if count == 1 && !is_p1 {
        // Check if p1 took a corner (indices 0, 3, 12, 15, 48, 51, 60, 63)
        let corners = 0x9009000000009009u64;
        if (p1 & corners) != 0 {
            // Return a center piece (e.g., 21, 22, 25, 26, 37, 38, 41, 42)
            // 21 is (1,1,1), 42 is (2,2,2)
            if (p1 & (1<<21)) == 0 { return Some(21); }
            if (p1 & (1<<42)) == 0 { return Some(42); }
        }
    }
    None
}

// --- Search Logic ---

#[wasm_bindgen]
pub fn search_depth(p1: u64, p2: u64, ai_is_p1: bool, depth: u8, stop_time: f64) -> JsValue {
    if js_sys::Date::now() > stop_time {
        return serialize_res(0, 0, true);
    }
    
    // Check Book Move
    if let Some(mv) = get_book_move(p1, p2, ai_is_p1) {
        return serialize_res(mv, WIN_SCORE, false);
    }

    NODE_COUNT.with(|c| *c.borrow_mut() = 0);
    
    // Check Forced Move at Root
    let my_mask = if ai_is_p1 { p1 } else { p2 };
    let opp_mask = if ai_is_p1 { p2 } else { p1 };
    
    if let Some(mv) = get_forced_move(my_mask, opp_mask) {
         return serialize_res(mv, WIN_SCORE, false);
    }

    // Standard Alpha-Beta
    let mut moves = [0u8; 64];
    let count = get_sorted_moves(p1, p2, ai_is_p1, &mut moves);
    
    if count == 0 { return JsValue::NULL; }

    let mut best_move = moves[0];
    let mut max_eval = -INF;
    let mut alpha = -INF;
    let beta = INF;
    let mut time_abort = false;

    for i in 0..count {
        let m = moves[i];
        let bit = 1u64 << m;
        let (np1, np2) = if ai_is_p1 { (p1 | bit, p2) } else { (p1, p2 | bit) };
        
        let score = -negamax(
            np1, np2, 
            (depth as i8) - 1, 
            -beta, -alpha, 
            !ai_is_p1, 
            stop_time, 
            &mut time_abort
        );

        if time_abort { break; }

        if score > max_eval {
            max_eval = score;
            best_move = m;
        }
        alpha = alpha.max(score);
    }

    serialize_res(best_move, max_eval, time_abort)
}

fn serialize_res(idx: u8, score: i32, abort: bool) -> JsValue {
    let r = SearchResult {
        x: idx % 4,
        z: (idx / 4) % 4,
        y: idx / 16,
        score,
        depth: 0,
        time_abort: abort
    };
    serde_wasm_bindgen::to_value(&r).unwrap()
}

fn negamax(
    p1: u64, p2: u64, depth: i8, 
    mut alpha: i32, mut beta: i32, 
    is_p1: bool, stop_time: f64, abort: &mut bool
) -> i32 {
    NODE_COUNT.with(|c| {
        let mut val = c.borrow_mut();
        *val += 1;
        if *val & TIME_CHECK_INTERVAL == 0 {
            if js_sys::Date::now() > stop_time { *abort = true; }
        }
    });
    if *abort { return 0; }

    let (me, opp) = if is_p1 { (p1, p2) } else { (p2, p1) };

    // Terminal Check
    if check_win(opp) { return -WIN_SCORE - (depth as i32); } // Prefer faster wins
    if depth <= 0 { return evaluate(me, opp); }

    // TT Lookup
    let orig_alpha = alpha;
    if let Some(entry) = TT.with(|tt| tt.borrow().get(&tt_key(p1, p2, is_p1)).copied()) {
        if entry.depth >= depth {
            match entry.flag {
                TT_EXACT => return entry.score,
                TT_LOWER => alpha = alpha.max(entry.score),
                TT_UPPER => beta = beta.min(entry.score),
                _ => {}
            }
            if alpha >= beta { return entry.score; }
        }
    }

    // Forced Move Pruning
    let mut moves = [0u8; 64];
    let mut count = 0;
    let mut forced = false;

    if let Some(fm) = get_forced_move(me, opp) {
        moves[0] = fm;
        count = 1;
        forced = true;
    } else {
        count = get_sorted_moves(p1, p2, is_p1, &mut moves);
    }

    if count == 0 { return 0; } // Draw or Stuck

    let mut val = -INF;
    let mut best_m = moves[0];

    for i in 0..count {
        let m = moves[i];
        let bit = 1u64 << m;
        let (np1, np2) = if is_p1 { (p1 | bit, p2) } else { (p1, p2 | bit) };
        
        // Extension: If forced move, do not reduce depth (or reduce less)
        // to ensure we see the result of the force sequence
        let d_reduction = if forced { 0 } else { 1 };
        
        let score = -negamax(np1, np2, depth - d_reduction, -beta, -alpha, !is_p1, stop_time, abort);
        
        if *abort { return 0; }

        if score > val {
            val = score;
            best_m = m;
        }
        alpha = alpha.max(score);
        if alpha >= beta { break; }
    }

    // TT Store
    let flag = if val <= orig_alpha { TT_UPPER }
               else if val >= beta { TT_LOWER }
               else { TT_EXACT };
    
    TT.with(|tt| {
        tt.borrow_mut().insert(tt_key(p1, p2, is_p1), TTEntry {
            depth, score: val, flag, best_move: best_m
        });
    });

    val
}

// --- Heuristics & Evaluation ---

fn check_win(mask: u64) -> bool {
    let lines = get_winning_lines();
    lines.iter().any(|&l| (mask & l) == l)
}

fn evaluate(my_mask: u64, opp_mask: u64) -> i32 {
    let lines = get_winning_lines();
    let mut score = 0;
    
    let mut my_forks = [0u8; 64];
    let mut opp_forks = [0u8; 64];

    for &line in lines.iter() {
        let my_c = (my_mask & line).count_ones();
        let opp_c = (opp_mask & line).count_ones();

        // Dead Tuple Check: If both have pieces, line is useless. Skip.
        if my_c > 0 && opp_c > 0 { continue; }

        if opp_c == 0 {
            // Live Tuple for Me
            match my_c {
                3 => score += 5000,
                2 => {
                    score += 200;
                    // Mark forks
                    let empty = line & !my_mask;
                    mark_forks(empty, &mut my_forks);
                },
                1 => score += 10,
                _ => {}
            }
        } else {
            // Live Tuple for Opponent
            match opp_c {
                3 => score -= 5000,
                2 => {
                    score -= 200;
                    let empty = line & !opp_mask;
                    mark_forks(empty, &mut opp_forks);
                },
                1 => score -= 10,
                _ => {}
            }
        }
    }

    // Fork Scoring
    for i in 0..64 {
        if my_forks[i] >= 2 { score += 3000; }
        if opp_forks[i] >= 2 { score -= 3000; }
        
        // Add Patashnik's Positional Weight for occupied cells
        if (my_mask & (1<<i)) != 0 { score += POSITIONAL_WEIGHTS[i]; }
        if (opp_mask & (1<<i)) != 0 { score -= POSITIONAL_WEIGHTS[i]; }
    }
    
    score
}

fn mark_forks(mut mask: u64, counts: &mut [u8; 64]) {
    while mask != 0 {
        let idx = mask.trailing_zeros();
        counts[idx as usize] += 1;
        mask &= !(1 << idx);
    }
}

fn get_sorted_moves(p1: u64, p2: u64, is_p1: bool, out: &mut [u8; 64]) -> usize {
    let occupied = p1 | p2;
    let mut count = 0;
    let mut scored_moves = [(0u8, 0i32); 64];
    
    // Get TT best move
    let best_m = TT.with(|tt| {
        tt.borrow().get(&tt_key(p1, p2, is_p1)).map(|e| e.best_move)
    });

    for i in 0..64 {
        if (occupied & (1 << i)) == 0 {
            let mut val = POSITIONAL_WEIGHTS[i];
            if Some(i as u8) == best_m { val += 100_000; }
            scored_moves[count] = (i as u8, val);
            count += 1;
        }
    }

    // Sort descending
    scored_moves[0..count].sort_unstable_by(|a, b| b.1.cmp(&a.1));
    for i in 0..count { out[i] = scored_moves[i].0; }
    
    count
}
```

### 2. The Config (`Cargo.toml`)

Ensure your dependencies match this to support the engine.

```toml
[package]
name = "qubic-engine"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
js-sys = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.4"
getrandom = { version = "0.2", features = ["js"] }
