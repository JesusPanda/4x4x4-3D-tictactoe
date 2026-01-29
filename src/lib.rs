use wasm_bindgen::prelude::*;
use serde::Serialize;
use std::sync::OnceLock;
use std::collections::HashMap;
use std::cell::RefCell;

// --- Constants & Precomputation ---

#[wasm_bindgen]
#[derive(Clone, Copy, Serialize)]
pub struct SearchResult {
    pub x: u8,
    pub y: u8,
    pub z: u8,
    pub score: i32,
    pub depth: u8,
    pub time_abort: bool, // New field to signal abort
}

// --- Static Data ---
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
    // 1. Rows
    for y in 0..4 {
        for z in 0..4 {
            let mut mask = 0;
            for x in 0..4 { mask |= cell_to_bit(x, y, z); }
            lines[idx] = mask; idx += 1;
        }
    }
    // 2. Columns
    for y in 0..4 {
        for x in 0..4 {
            let mut mask = 0;
            for z in 0..4 { mask |= cell_to_bit(x, y, z); }
            lines[idx] = mask; idx += 1;
        }
    }
    // 3. Pillars
    for x in 0..4 {
        for z in 0..4 {
            let mut mask = 0;
            for y in 0..4 { mask |= cell_to_bit(x, y, z); }
            lines[idx] = mask; idx += 1;
        }
    }
    // 4. Diagonals XZ
    for y in 0..4 {
        let mut m1 = 0; let mut m2 = 0;
        for i in 0..4 {
            m1 |= cell_to_bit(i, y, i);
            m2 |= cell_to_bit(3 - i, y, i);
        }
        lines[idx] = m1; idx += 1;
        lines[idx] = m2; idx += 1;
    }
    // 5. Diagonals XY
    for z in 0..4 {
        let mut m1 = 0; let mut m2 = 0;
        for i in 0..4 {
            m1 |= cell_to_bit(i, i, z);
            m2 |= cell_to_bit(i, 3 - i, z);
        }
        lines[idx] = m1; idx += 1;
        lines[idx] = m2; idx += 1;
    }
    // 6. Diagonals YZ
    for x in 0..4 {
        let mut m1 = 0; let mut m2 = 0;
        for i in 0..4 {
            m1 |= cell_to_bit(x, i, i);
            m2 |= cell_to_bit(x, i, 3 - i);
        }
        lines[idx] = m1; idx += 1;
        lines[idx] = m2; idx += 1;
    }
    // 7. 3D Main
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
        let dx = (2 * x - 3).abs();
        let dy = (2 * y - 3).abs();
        let dz = (2 * z - 3).abs();
        scored[i as usize] = (i, dx + dy + dz);
    }
    scored.sort_by_key(|s| s.1);
    let mut order = [0u8; 64];
    for (i, (cell_idx, _)) in scored.iter().enumerate() {
        order[i] = *cell_idx;
    }
    order
}

// --- Transposition Table ---
const TT_EXACT: u8 = 0;
const TT_LOWER: u8 = 1;
const TT_UPPER: u8 = 2;

#[derive(Clone, Copy)]
struct TTEntry {
    depth: i8,
    score: i32,
    flag: u8,
    best_move: u8,
}

thread_local! {
    static TT: RefCell<HashMap<u128, TTEntry>> = RefCell::new(HashMap::with_capacity(500_000));
}

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
        let key = tt_key(p1, p2, is_p1_turn);
        if let Some(existing) = table.get(&key) {
            if existing.depth > entry.depth {
                return;
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
const TIME_CHECK_INTERVAL: u32 = 4095;

thread_local! {
    static NODE_COUNT: RefCell<u32> = const { RefCell::new(0) };
}

/// Runs ONE depth of the search.
/// Returns SearchResult. If time_abort is true, score is invalid.
#[wasm_bindgen]
pub fn search_depth(
    p1_mask: u64,
    p2_mask: u64,
    ai_is_p1: bool,
    depth: u8,
    stop_time: f64
) -> JsValue {
    // Check time immediately
    if js_sys::Date::now() > stop_time {
        return serialize_result(None, 0, depth, true);
    }

    NODE_COUNT.with(|c| *c.borrow_mut() = 0);

    let mut alpha = -INF;
    let beta = INF;
    let mut current_best: Option<u8> = None;
    let mut max_eval = -INF;
    let mut time_abort = false;

    // Generate moves
    let mut moves = [0u8; 64];
    let move_count = get_available_moves(p1_mask, p2_mask, &mut moves);
    
    if move_count == 0 {
        return JsValue::NULL; // No moves ?
    }
    
    // Order moves (Try TT best first)
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

        // Root negamax
        let score = -negamax(
            new_p1, new_p2,
            (depth as i8) - 1,
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
    }

    serialize_result(current_best, max_eval, depth, time_abort)
}

fn serialize_result(idx: Option<u8>, score: i32, depth: u8, time_abort: bool) -> JsValue {
    if let Some(i) = idx {
        let x = (i % 4) as u8;
        let z = ((i / 4) % 4) as u8;
        let y = (i / 16) as u8;
        let m = SearchResult { x, y, z, score, depth, time_abort };
        serde_wasm_bindgen::to_value(&m).unwrap()
    } else {
        // Return null or partial struct? 
        // If abort and no move found yet (rare at root), or no moves available
        if time_abort {
             let m = SearchResult { x:0, y:0, z:0, score:0, depth, time_abort: true };
             serde_wasm_bindgen::to_value(&m).unwrap()
        } else {
            JsValue::NULL
        }
    }
}

fn order_moves(moves: &[u8; 64], count: usize, tt_best: Option<u8>, out: &mut [u8; 64]) {
    let mut write_idx = 0;
    if let Some(best) = tt_best {
        for i in 0..count {
            if moves[i] == best {
                out[write_idx] = best;
                write_idx += 1;
                break;
            }
        }
    }
    for i in 0..count {
        if Some(moves[i]) != tt_best {
            out[write_idx] = moves[i];
            write_idx += 1;
        }
    }
}

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

fn negamax(
    p1: u64, p2: u64,
    depth: i8,
    mut alpha: i32, mut beta: i32,
    is_p1_turn: bool,
    stop_time: f64,
    time_abort: &mut bool
) -> i32 {
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

    if check_win(opp_mask) {
        return -WIN_SCORE - (depth as i32);
    }

    let orig_alpha = alpha;
    let mut tt_best_move: Option<u8> = None;

    if let Some(entry) = tt_lookup(p1, p2, is_p1_turn) {
        tt_best_move = Some(entry.best_move);
        if entry.depth >= depth {
            match entry.flag {
                TT_EXACT => return entry.score,
                TT_LOWER => alpha = alpha.max(entry.score),
                TT_UPPER => beta = beta.min(entry.score),
                _ => {}
            }
            if alpha >= beta {
                return entry.score;
            }
        }
    }

    if depth <= 0 {
        return evaluate(my_mask, opp_mask);
    }

    let mut moves = [0u8; 64];
    let move_count = get_available_moves(p1, p2, &mut moves);
    if move_count == 0 { return 0; }

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
            break; 
        }
    }

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
        if (mask & line) == line { return true; }
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
            score += match my_count { 3 => 1000, 2 => 50, 1 => 5, _ => 0 };
        } else if my_count == 0 {
            score -= match opp_count { 3 => 1000, 2 => 50, 1 => 5, _ => 0 };
        }
    }
    score
}
