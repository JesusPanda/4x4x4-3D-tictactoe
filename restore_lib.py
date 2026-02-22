import sys

with open('src/lib.rs', 'r') as f:
    lines = f.readlines()

idx_start = -1
idx_end = -1

for i, line in enumerate(lines):
    if '// Fork Scoring' in line:
        idx_start = i
    if '// Sort descending' in line:
        idx_end = i

if idx_start != -1 and idx_end != -1:
    new_lines = lines[:idx_start]

    # Restored content
    restored = [
        "    // Fork Scoring\n",
        "    for i in 0..64 {\n",
        "        if my_forks[i] >= 2 { score += 3000; }\n",
        "        if opp_forks[i] >= 2 { score -= 3000; }\n",
        "        \n",
        "        // Add Patashnik's Positional Weight for occupied cells\n",
        "        if (my_mask & (1<<i)) != 0 { score += POSITIONAL_WEIGHTS[i]; }\n",
        "        if (opp_mask & (1<<i)) != 0 { score -= POSITIONAL_WEIGHTS[i]; }\n",
        "    }\n",
        "    \n",
        "    score\n",
        "}\n",
        "\n",
        "fn mark_forks(mut mask: u64, counts: &mut [u8; 64]) {\n",
        "    while mask != 0 {\n",
        "        let idx = mask.trailing_zeros();\n",
        "        counts[idx as usize] += 1;\n",
        "        mask &= !(1 << idx);\n",
        "    }\n",
        "}\n",
        "\n",
        "fn get_sorted_moves(p1: u64, p2: u64, is_p1: bool, out: &mut [u8; 64]) -> usize {\n",
        "    let occupied = p1 | p2;\n",
        "    let mut count = 0;\n",
        "    let mut scored_moves = [(0u8, 0i32); 64];\n",
        "    \n",
        "    // Get TT best move\n",
        "    let best_m = TT.with(|tt| {\n",
        "        tt.borrow().get(&tt_key(p1, p2, is_p1)).map(|e| e.best_move)\n",
        "    });\n",
        "\n",
        "    let mut empty = !occupied;\n",
        "    while empty != 0 {\n",
        "        let i = empty.trailing_zeros();\n",
        "        empty &= empty - 1;\n",
        "\n",
        "        let mut val = POSITIONAL_WEIGHTS[i as usize];\n",
        "        if Some(i as u8) == best_m { val += 100_000; }\n",
        "        scored_moves[count] = (i as u8, val);\n",
        "        count += 1;\n",
        "    }\n",
        "\n"
    ]

    new_lines.extend(restored)
    new_lines.extend(lines[idx_end:])

    with open('src/lib.rs', 'w') as f:
        f.writelines(new_lines)
    print("Successfully restored src/lib.rs")
else:
    print("Could not find markers")
