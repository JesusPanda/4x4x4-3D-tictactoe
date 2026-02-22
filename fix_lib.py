import sys

with open('src/lib.rs', 'r') as f:
    lines = f.readlines()

new_lines = []
skip = False
for line in lines:
    if 'let mut empty = !occupied;' in line:
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
    elif skip:
        # Skip until we find the end of the messed up block
        # The messed up block ends at '    }' before '// Sort descending'
        if '    // Sort descending' in line:
            skip = False
            new_lines.append('\n')
            new_lines.append(line)
    else:
        new_lines.append(line)

with open('src/lib.rs', 'w') as f:
    f.writelines(new_lines)
