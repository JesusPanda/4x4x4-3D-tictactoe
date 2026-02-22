import sys

with open('src/lib.rs', 'r') as f:
    lines = f.readlines()

new_lines = []
in_main_func = False
in_baseline_func = False
skip = False

for line in lines:
    stripped = line.strip()

    # Detect function start
    if 'fn get_sorted_moves(' in line:
        in_main_func = True
        in_baseline_func = False
    elif 'fn get_sorted_moves_baseline(' in line:
        in_baseline_func = True
        in_main_func = False

    # Detect the loop start (which is now messed up or modified)
    # The 'sed' replaced 'for i in 0..64 {' with 'let mut empty = !occupied;\n    while empty != 0 {'
    # So we look for 'let mut empty = !occupied;'

    if 'let mut empty = !occupied;' in line:
        if in_main_func:
            # Clean Optimized Loop
            new_lines.append('    let mut empty = !occupied;\n')
            new_lines.append('    while empty != 0 {\n')
            new_lines.append('        let i = empty.trailing_zeros();\n')
            new_lines.append('        empty &= empty - 1;\n')
            new_lines.append('\n')
            new_lines.append('        let mut val = POSITIONAL_WEIGHTS[i as usize];\n')
            new_lines.append('        if Some(i as u8) == best_m { val += 100_000; }\n')
            new_lines.append('        scored_moves[count] = (i as u8, val);\n')
            new_lines.append('        count += 1;\n')
            new_lines.append('    }\n')
            skip = True
        elif in_baseline_func:
            # Restore Original Loop (Baseline)
            new_lines.append('        for i in 0..64 {\n')
            new_lines.append('            if (occupied & (1 << i)) == 0 {\n')
            new_lines.append('                let mut val = POSITIONAL_WEIGHTS[i];\n')
            new_lines.append('                if Some(i as u8) == best_m { val += 100_000; }\n')
            new_lines.append('                scored_moves[count] = (i as u8, val);\n')
            new_lines.append('                count += 1;\n')
            new_lines.append('            }\n')
            new_lines.append('        }\n')
            skip = True
        else:
            # Should not happen if structured correctly, but maybe inside another func?
            new_lines.append(line)

    elif skip:
        # We need to skip the messed up block until '// Sort descending'
        if '// Sort descending' in line:
            skip = False
            new_lines.append('\n')
            new_lines.append(line)
    else:
        new_lines.append(line)

with open('src/lib.rs', 'w') as f:
    f.writelines(new_lines)
