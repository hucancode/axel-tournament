#!/bin/bash
set -e

cd /workspace

# Function to detect file language
detect_language() {
    local file=$1
    case "${file##*.}" in
        rs) echo "rust" ;;
        go) echo "go" ;;
        c) echo "c" ;;
        cpp|cc|cxx) echo "cpp" ;;
        py) echo "python" ;;
        *) echo "unknown" ;;
    esac
}

# Function to compile code based on language
compile_code() {
    local source=$1
    local output=$2
    local lang=$(detect_language "$source")

    case "$lang" in
        rust)
            rustc --edition 2021 "$source" -C opt-level=2 -o "$output" 2>&1 || return 1
            ;;
        go)
            go build -o "$output" "$source" 2>&1 || return 1
            ;;
        c)
            gcc "$source" -o "$output" -O2 2>&1 || return 1
            ;;
        cpp)
            g++ "$source" -o "$output" -O2 -std=c++17 2>&1 || return 1
            ;;
        python)
            # Python doesn't need compilation, just make executable
            cp "$source" "$output"
            chmod +x "$output"
            # Add shebang if not present
            if ! head -n 1 "$output" | grep -q "^#!"; then
                echo -e "#!/usr/bin/env python3\n$(cat $output)" > "$output"
            fi
            return 0
            ;;
        *)
            echo "Unknown language for $source"
            return 1
            ;;
    esac
}

# Find server code (server.rs, server.go, server.c, server.py, etc.)
SERVER_FILE=""
for ext in rs go c cpp py; do
    if [ -f "server.$ext" ]; then
        SERVER_FILE="server.$ext"
        break
    fi
done

if [ -z "$SERVER_FILE" ]; then
    echo "RE RE"
    exit 1
fi

# Compile server
echo "Compiling server: $SERVER_FILE"
# If a Cargo.toml already exists (user provided), use cargo build; otherwise compile directly
if [ -f "Cargo.toml" ] && [[ "$SERVER_FILE" == *.rs ]]; then
    echo "Using cargo build for server with dependencies (offline)"
    if ! CARGO_NET_OFFLINE=1 cargo build --offline --release --bin game_server 2>&1; then
        echo "RE RE"
        exit 1
    fi
    cp target/release/game_server server
elif ! compile_code "$SERVER_FILE" "server"; then
    echo "RE RE"
    exit 1
fi
chmod +x server

# Compile all player code files
PLAYERS=()
PLAYER_COUNT=0
PLAYER_COMPILE_FAILED=false

for file in player_*.*; do
    if [ -f "$file" ] && [[ "$file" != *.toml ]]; then
        idx=$(echo "$file" | sed 's/player_//; s/\.[^.]*$//')
        binary="player_${idx}"

        echo "Compiling player $idx: $file"
        if compile_code "$file" "$binary"; then
            chmod +x "$binary"
            PLAYERS+=("/workspace/$binary")
            PLAYER_COUNT=$((PLAYER_COUNT + 1))
        else
            # Player compilation failed
            PLAYER_COMPILE_FAILED=true
            PLAYERS+=("__CE__${idx}")
            PLAYER_COUNT=$((PLAYER_COUNT + 1))
        fi
    fi
done

# Handle compilation errors
if [ $PLAYER_COUNT -eq 0 ]; then
    echo "RE RE"
    exit 1
fi

# If any player failed to compile, emit CE tokens and exit with code 2
if [ "$PLAYER_COMPILE_FAILED" = true ]; then
    OUTPUT=""
    for player in "${PLAYERS[@]}"; do
        if [[ "$player" == __CE__* ]]; then
            OUTPUT="$OUTPUT CE"
        else
            OUTPUT="$OUTPUT 0"
        fi
    done
    echo "${OUTPUT# }"
    exit 2
fi

# Run the game server with player binaries
# Filter out CE placeholders
VALID_PLAYERS=()
for player in "${PLAYERS[@]}"; do
    if [[ "$player" != __CE__* ]]; then
        VALID_PLAYERS+=("$player")
    fi
done

# Detect if server is Python
if [[ "$SERVER_FILE" == *.py ]]; then
    python3 server "${VALID_PLAYERS[@]}" 2>&1
else
    ./server "${VALID_PLAYERS[@]}" 2>&1
fi
