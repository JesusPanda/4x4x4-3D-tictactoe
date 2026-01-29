use wasm_bindgen::prelude::*;
use serde::Serialize;
use std::sync::OnceLock;
use std::collections::HashMap;
use std::cell::RefCell;

// --- Constants & Precomputation ---

// Bitboard representation: u64
// 4x4x4 grid = 64 cells.
// x, y, z -> bit index = y*16 + z*4 + x

#[wasm_bindgen]
#[derive(Clone, Copy, Serialize)]
pub struct Move {
    pub x: u8,
    pub y: u8,
    pub z: u8,
    pub score: i32,
}

#[wasm_bindgen]
#[derive(Clone, Copy, Serialize)]
pub struct SearchResult {
    pub x: u8,
    pub y: u8,
    pub z: u8,
    pub score: i32,
    pub depth: u8,
}

// --- Static Data (OnceLock for safe initialization) ---

static WINNING_LINES: OnceLock<[u64; 76]> = OnceLock::new();
static MOVE_ORDER: OnceLock<[u8; 64]> = OnceLock::new();

fn get_winning_lines() -> &'static [u64; 76] {
    WINNING_LINES.get_or_init(generate_winning_lines)
}

fn get_move_order() -> &'static [u8; 64] {
    MOVE_ORDER.get_or_init(generate_move_order)
}

fn generate_winning_lines() -> [u64; 76] {
    let mut lines = [0u64; 76];
    let mut idx = 0;
    let cell_to_bit = |x: usize, y: usize, z: usize| 1u64 << (y * 16 + z * 4 + x);

    // 1. Rows (along X) - 16 lines
    for y in 0..4 {
        for z in 0..4 {
            let mut mask = 0;
            for x in 0..4 { mask |= cell_to_bit(x, y, z); }
            lines[idx] = mask;
            idx += 1;
        }
    }

    // 2. Columns (along Z) - 16 lines
    for y in 0..4 {
        for x in 0..4 {
            let mut mask = 0;
            for z in 0..4 { mask |= cell_to_bit(x, y, z); }
            lines[idx] = mask;
            idx += 1;
        }
    }

    // 3. Pillars (along Y) - 16 lines
    for x in 0..4 {
        for z in 0..4 {
            let mut mask = 0;
            for y in 0..4 { mask |= cell_to_bit(x, y, z); }
            lines[idx] = mask;
            idx += 1;
        }
    }

    // 4. 2D Diagonals XZ (fixed Y) - 8 lines
    for y in 0..4 {
        let mut m1 = 0; let mut m2 = 0;
        for i in 0..4 {
            m1 |= cell_to_bit(i, y, i);
            m2 |= cell_to_bit(3 - i, y, i);
        }
        lines[idx] = m1; idx += 1;
        lines[idx] = m2; idx += 1;
    }

    // 5. 2D Diagonals XY (fixed Z) - 8 lines
    for z in 0..4 {
        let mut m1 = 0; let mut m2 = 0;
        for i in 0..4 {
            m1 |= cell_to_bit(i, i, z);
            m2 |= cell_to_bit(i, 3 - i, z);
        }
        lines[idx] = m1; idx += 1;
        lines[idx] = m2; idx += 1;
    }

    // 6. 2D Diagonals YZ (fixed X) - 8 lines
    for x in 0..4 {
        let mut m1 = 0; let mut m2 = 0;
        for i in 0..4 {
            m1 |= cell_to_bit(x, i, i);
            m2 |= cell_to_bit(x, i, 3 - i);
        }
        lines[idx] = m1; idx += 1;
        lines[idx] = m2; idx += 1;
    }

    // 7. 3D Main Diagonals - 4 lines
    let mut d1 = 0; let mut d2 = 0; let mut d3 = 0; let mut d4 = 0;
    for i in 0..4 {
        d1 |= cell_to_bit(i, i, i);
        d2 |= cell_to_bit(3 - i, i, i);
        d3 |= cell_to_bit(i, i, 3 - i);
        d4 |= cell_to_bit(3 - i, i, 3 - i);
    }
    lines[idx] = d1; idx += 1;
    lines[idx] = d2; idx += 1;
    lines[idx] = d3; idx += 1;
    lines[idx] = d4;

    lines
}

fn generate_move_order() -> [u8; 64] {
    let mut scored: [(u8, i32); 64] = [(0, 0); 64];
    for i in 0..64u8 {
        let x = (i % 4) as i32;
        let z = ((i / 4) % 4) as i32;
        let y = (i / 16) as i32;
        // Manhattan distance from center (1.5, 1.5, 1.5), scaled by 2 to avoid floats
        let dx = (2 * x - 3).abs();
        let dy = (2 * y - 3).abs();
        let dz = (2 * z - 3).abs();
        scored[i as usize] = (i, dx + dy + dz);
    }

    // Sort by distance (ascending)
    scored.sort_by_key(|s| s.1);

    let mut order = [0u8; 64];
    for (i, (cell_idx, _)) in scored.iter().enumerate() {
        order[i] = *cell_idx;
    }
    order
}

// --- Transposition Table ---

const TT_EXACT: u8 = 0;
const TT_LOWER: u8 = 1; // Alpha cutoff (score >= beta)
const TT_UPPER: u8 = 2; // Beta cutoff (score <= alpha)

#[derive(Clone, Copy)]
struct TTEntry {
    depth: i8,
    score: i32,
    flag: u8,
    best_move: u8, // Index 0-63
}

// Thread-local TT for Wasm (single-threaded)
// We persist TT across searches for better performance
thread_local! {
    static TT: RefCell<HashMap<u128, TTEntry>> = RefCell::new(HashMap::with_capacity(500_000));
}

/// TT key includes whose turn it is to avoid collisions
fn tt_key(p1: u64, p2: u64, is_p1_turn: bool) -> u128 {
    let turn_bit = if is_p1_turn { 1u128 } else { 0u128 };
    ((p1 as u128) << 65) | ((p2 as u128) << 1) | turn_bit
}

fn tt_lookup(p1: u64, p2: u64, is_p1_turn: bool) -> Option<TTEntry> {
    TT.with(|tt| tt.borrow().get(&tt_key(p1, p2, is_p1_turn)).copied())
}

fn tt_store(p1: u64, p2: u64, is_p1_turn: bool, entry: TTEntry) {
    TT.with(|tt| {
        let mut table = tt.borrow_mut();
        // Replace if new entry has greater or equal depth
        let key = tt_key(p1, p2, is_p1_turn);
        if let Some(existing) = table.get(&key) {
            if existing.depth > entry.depth {
                return; // Keep deeper entry
            }
        }
        table.insert(key, entry);
    });
}

#[wasm_bindgen]
pub fn clear_tt() {
    TT.with(|tt| tt.borrow_mut().clear());
}

// --- Search Logic ---

const INF: i32 = 1_000_000;
const WIN_SCORE: i32 = 1_000_000;
const TIME_CHECK_INTERVAL: u32 = 4095; // Check time every 4096 nodes

// Thread-local state for search
thread_local! {
    static NODE_COUNT: RefCell<u32> = const { RefCell::new(0) };
}

/// Main entry point - returns result with depth info for progress display
#[wasm_bindgen]
pub fn get_best_move(
    p1_mask: u64,
    p2_mask: u64,
    ai_is_p1: bool,
    time_limit_ms: f64,
    progress_callback: &js_sys::Function
) -> JsValue {
    NODE_COUNT.with(|c| *c.borrow_mut() = 0);

    let start_time = js_sys::Date::now();
    let stop_time = start_time + time_limit_ms;

    let mut best_move_idx: Option<u8> = None;
    let mut best_score = -INF;
    let mut best_depth: u8 = 0;

    // Stack-allocated move buffer
    let mut moves = [0u8; 64];
    let move_count = get_available_moves(p1_mask, p2_mask, &mut moves);

    if move_count == 0 {
        return JsValue::NULL;
    }

    // Iterative Deepening
    for depth in 1i8..=14 {
        if js_sys::Date::now() > stop_time { break; }

        let mut alpha = -INF;
        let mut beta = INF;
        let mut current_best: Option<u8> = None;
        let mut max_eval = -INF;
        let mut time_abort = false;

        // Try TT best move first if available
        let mut ordered_moves = [0u8; 64];
        let tt_best = tt_lookup(p1_mask, p2_mask, ai_is_p1).map(|e| e.best_move);
        order_moves(&moves, move_count, tt_best, &mut ordered_moves);

        for i in 0..move_count {
            let m_idx = ordered_moves[i];
            let bit = 1u64 << m_idx;
            let (new_p1, new_p2) = if ai_is_p1 {
                (p1_mask | bit, p2_mask)
            } else {
                (p1_mask, p2_mask | bit)
            };

            let score = -negamax(
                new_p1, new_p2,
                depth - 1,
                -beta, -alpha,
                !ai_is_p1,
                stop_time,
                &mut time_abort
            );

            if time_abort { break; }

            if score > max_eval {
                max_eval = score;
                current_best = Some(m_idx);
            }
            alpha = alpha.max(score);

            // Found forced win
            if score >= WIN_SCORE - 100 {
                return serialize_result(Some(m_idx), score, depth as u8);
            }
        }

        if !time_abort {
            if let Some(m) = current_best {
                best_move_idx = Some(m);
                best_score = max_eval;
                best_depth = depth as u8;

                // Report progress to JS
                let _ = progress_callback.call2(
                    &JsValue::NULL,
                    &JsValue::from(depth),
                    &JsValue::from(max_eval)
                );
            }
        }
    }

    serialize_result(best_move_idx, best_score, best_depth)
}

/// Order moves with TT best move first
fn order_moves(moves: &[u8; 64], count: usize, tt_best: Option<u8>, out: &mut [u8; 64]) {
    let mut write_idx = 0;

    // TT best move first
    if let Some(best) = tt_best {
        for i in 0..count {
            if moves[i] == best {
                out[write_idx] = best;
                write_idx += 1;
                break;
            }
        }
    }

    // Rest of moves
    for i in 0..count {
        if Some(moves[i]) != tt_best {
            out[write_idx] = moves[i];
            write_idx += 1;
        }
    }
}

fn serialize_result(idx: Option<u8>, score: i32, depth: u8) -> JsValue {
    if let Some(i) = idx {
        let x = (i % 4) as u8;
        let z = ((i / 4) % 4) as u8;
        let y = (i / 16) as u8;
        let m = SearchResult { x, y, z, score, depth };
        serde_wasm_bindgen::to_value(&m).unwrap()
    } else {
        JsValue::NULL
    }
}

/// Fills `moves` buffer with available move indices, returns count
fn get_available_moves(p1: u64, p2: u64, moves: &mut [u8; 64]) -> usize {
    let occupied = p1 | p2;
    let order = get_move_order();
    let mut count = 0;
    for &idx in order.iter() {
        if (occupied & (1u64 << idx)) == 0 {
            moves[count] = idx;
            count += 1;
        }
    }
    count
}

/// NegaMax with Alpha-Beta and Transposition Table
fn negamax(
    p1: u64, p2: u64,
    depth: i8,
    mut alpha: i32, mut beta: i32,
    is_p1_turn: bool,
    stop_time: f64,
    time_abort: &mut bool
) -> i32 {
    // Throttled time check (every 4096 nodes)
    NODE_COUNT.with(|c| {
        let mut count = c.borrow_mut();
        *count += 1;
        if *count & TIME_CHECK_INTERVAL == 0 {
            if js_sys::Date::now() > stop_time {
                *time_abort = true;
            }
        }
    });
    if *time_abort { return 0; }

    let my_mask = if is_p1_turn { p1 } else { p2 };
    let opp_mask = if is_p1_turn { p2 } else { p1 };

    // Terminal: opponent won (they just moved)
    if check_win(opp_mask) {
        return -WIN_SCORE - (depth as i32);
    }

    // Transposition Table lookup
    let orig_alpha = alpha;
    let mut tt_best_move: Option<u8> = None;

    if let Some(entry) = tt_lookup(p1, p2, is_p1_turn) {
        tt_best_move = Some(entry.best_move);
        if entry.depth >= depth {
            match entry.flag {
                TT_EXACT => return entry.score,
                TT_LOWER => alpha = alpha.max(entry.score),
                TT_UPPER => beta = beta.min(entry.score), // FIXED: Now actually uses upper bound
                _ => {}
            }
            if alpha >= beta {
                return entry.score;
            }
        }
    }

    // Leaf node
    if depth <= 0 {
        let score = evaluate(my_mask, opp_mask);
        return score;
    }

    // Generate moves (stack allocated)
    let mut moves = [0u8; 64];
    let move_count = get_available_moves(p1, p2, &mut moves);

    if move_count == 0 {
        return 0; // Draw
    }

    // Order moves with TT best first
    let mut ordered_moves = [0u8; 64];
    order_moves(&moves, move_count, tt_best_move, &mut ordered_moves);

    let mut max_val = -INF;
    let mut best_move: u8 = ordered_moves[0];

    for i in 0..move_count {
        let m_idx = ordered_moves[i];
        let bit = 1u64 << m_idx;
        let (next_p1, next_p2) = if is_p1_turn {
            (p1 | bit, p2)
        } else {
            (p1, p2 | bit)
        };

        let val = -negamax(
            next_p1, next_p2,
            depth - 1,
            -beta, -alpha,
            !is_p1_turn,
            stop_time,
            time_abort
        );

        if *time_abort { return 0; }

        if val > max_val {
            max_val = val;
            best_move = m_idx;
        }
        alpha = alpha.max(val);
        if alpha >= beta {
            break; // Beta cutoff
        }
    }

    // Store in TT
    let flag = if max_val <= orig_alpha {
        TT_UPPER
    } else if max_val >= beta {
        TT_LOWER
    } else {
        TT_EXACT
    };
    tt_store(p1, p2, is_p1_turn, TTEntry { depth, score: max_val, flag, best_move });

    max_val
}

#[inline]
fn check_win(mask: u64) -> bool {
    let lines = get_winning_lines();
    for &line in lines.iter() {
        if (mask & line) == line {
            return true;
        }
    }
    false
}

fn evaluate(my_mask: u64, opp_mask: u64) -> i32 {
    let lines = get_winning_lines();
    let mut score = 0i32;

    for &line in lines.iter() {
        let my_count = (my_mask & line).count_ones();
        let opp_count = (opp_mask & line).count_ones();

        if my_count == 4 { return WIN_SCORE; }
        if opp_count == 4 { return -WIN_SCORE; }

        if opp_count == 0 {
            score += match my_count {
                3 => 1000,
                2 => 50,
                1 => 5,
                _ => 0,
            };
        } else if my_count == 0 {
            score -= match opp_count {
                3 => 1000,
                2 => 50,
                1 => 5,
                _ => 0,
            };
        }
    }
    score
}
