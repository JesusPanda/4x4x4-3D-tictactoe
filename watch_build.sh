#!/bin/bash
echo "Watching src/ for changes..."
while inotifywait -r -e close_write src/; do
    echo "Change detected in src/, rebuilding..."
    wasm-pack build --target web
    echo "Build complete. Resuming watch."
done
