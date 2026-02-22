import sys

with open('src/lib.rs', 'r') as f:
    lines = f.readlines()

# Find end of get_sorted_moves
idx_end_func = -1
for i, line in enumerate(lines):
    if '// Sort descending' in line:
        # The function ends a few lines after this
        # We look for the next '}' at indentation level 0 (or 1 depending on file)
        # Actually, get_sorted_moves is top level fn.
        # But let's look for '    count\n' then '}\n'
        for j in range(i, len(lines)):
            if lines[j].strip() == '}':
                idx_end_func = j
                break
        break

if idx_end_func != -1:
    new_lines = lines[:idx_end_func+1]
    new_lines.append('\n')

    # Append the test module
    test_mod = [
        "#[cfg(test)]\n",
        "mod bench_tests {\n",
        "    use super::*;\n",
        "    use std::time::Instant;\n",
        "\n",
        "    fn get_sorted_moves_baseline(p1: u64, p2: u64, is_p1: bool, out: &mut [u8; 64]) -> usize {\n",
        "        let occupied = p1 | p2;\n",
        "        let mut count = 0;\n",
        "        let mut scored_moves = [(0u8, 0i32); 64];\n",
        "        \n",
        "        // Get TT best move\n",
        "        let best_m = TT.with(|tt| {\n",
        "            tt.borrow().get(&tt_key(p1, p2, is_p1)).map(|e| e.best_move)\n",
        "        });\n",
        "\n",
        "        for i in 0..64 {\n",
        "            if (occupied & (1 << i)) == 0 {\n",
        "                let mut val = POSITIONAL_WEIGHTS[i];\n",
        "                if Some(i as u8) == best_m { val += 100_000; }\n",
        "                scored_moves[count] = (i as u8, val);\n",
        "                count += 1;\n",
        "            }\n",
        "        }\n",
        "\n",
        "        // Sort descending\n",
        "        scored_moves[0..count].sort_unstable_by(|a, b| b.1.cmp(&a.1));\n",
        "        for i in 0..count { out[i] = scored_moves[i].0; }\n",
        "        \n",
        "        count\n",
        "    }\n",
        "\n",
        "    #[test]\n",
        "    fn test_correctness() {\n",
        "        let p1_cases = [0u64, 0xFFFFFFFF00000000, 0xAAAAAAAAAAAAAAAA];\n",
        "        let p2_cases = [0u64, 0x00000000FFFFFFFF, 0x5555555555555555];\n",
        "\n",
        "        for &p1 in &p1_cases {\n",
        "            for &p2 in &p2_cases {\n",
        "                for &is_p1 in &[true, false] {\n",
        "                    let mut moves_opt = [0u8; 64];\n",
        "                    let mut moves_base = [0u8; 64];\n",
        "                    \n",
        "                    let count_opt = get_sorted_moves(p1, p2, is_p1, &mut moves_opt);\n",
        "                    let count_base = get_sorted_moves_baseline(p1, p2, is_p1, &mut moves_base);\n",
        "                    \n",
        "                    assert_eq!(count_opt, count_base, \"Count mismatch for p1={:x}, p2={:x}\", p1, p2);\n",
        "                    assert_eq!(moves_opt[..count_opt], moves_base[..count_base], \"Moves mismatch for p1={:x}, p2={:x}\", p1, p2);\n",
        "                }\n",
        "            }\n",
        "        }\n",
        "    }\n",
        "\n",
        "    #[test]\n",
        "    fn bench_performance() {\n",
        "        let p1 = 0x1234567890ABCDEF;\n",
        "        let p2 = 0xFEDCBA0987654321;\n",
        "        let is_p1 = true;\n",
        "        let mut moves = [0u8; 64];\n",
        "        \n",
        "        // Warmup\n",
        "        for _ in 0..100 {\n",
        "            get_sorted_moves(p1, p2, is_p1, &mut moves);\n",
        "            get_sorted_moves_baseline(p1, p2, is_p1, &mut moves);\n",
        "        }\n",
        "\n",
        "        let iterations = 200_000;\n",
        "\n",
        "        let start_base = Instant::now();\n",
        "        for _ in 0..iterations {\n",
        "            get_sorted_moves_baseline(p1, p2, is_p1, &mut moves);\n",
        "        }\n",
        "        let dur_base = start_base.elapsed();\n",
        "\n",
        "        let start_opt = Instant::now();\n",
        "        for _ in 0..iterations {\n",
        "            get_sorted_moves(p1, p2, is_p1, &mut moves);\n",
        "        }\n",
        "        let dur_opt = start_opt.elapsed();\n",
        "\n",
        "        println!(\"Baseline: {:?}\", dur_base);\n",
        "        println!(\"Optimized: {:?}\", dur_opt);\n",
        "    }\n",
        "}\n"
    ]

    new_lines.extend(test_mod)

    with open('src/lib.rs', 'w') as f:
        f.writelines(new_lines)
    print("Successfully restored tests in src/lib.rs")
else:
    print("Could not find end of get_sorted_moves")
