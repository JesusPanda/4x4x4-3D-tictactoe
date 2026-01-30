https://jesuspanda.github.io/4x4x4-3D-tictactoe/

# Development

To build the project, you need `wasm-pack`.

## Build Pipeline

To automatically rebuild the project whenever you modify files in `src/`, run:

```bash
./watch_build.sh
```

This script watches the `src/` directory and runs `wasm-pack build --target web` on any change.

**Note:** The script requires `inotify-tools` (Linux).
