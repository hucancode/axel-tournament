#!/bin/bash
set -e

OUTPUT_DIR="${OUTPUT_DIR:-$PWD}"

mkdir -p "${OUTPUT_DIR}"

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

SERVER_BIN="${OUTPUT_DIR}/server"
# Compile server (single-file source only; no Cargo dependencies)
echo "Compiling server: $SERVER_FILE"
if ! compile_code "$SERVER_FILE" "$SERVER_BIN"; then
    echo "RE RE"
    exit 1
fi
chmod +x "$SERVER_BIN"

# Compile all player code files
PLAYERS=()
PLAYER_COUNT=0
PLAYER_COMPILE_FAILED=false

for file in player_*.*; do
    if [ -f "$file" ] && [[ "$file" != *.toml ]]; then
        idx=$(echo "$file" | sed 's/player_//; s/\.[^.]*$//')
        binary="player_${idx}"
        output="${OUTPUT_DIR}/${binary}"

        echo "Compiling player $idx: $file"
        if compile_code "$file" "$output"; then
            chmod +x "$output"
            PLAYERS+=("$output")
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

"$SERVER_BIN" "${VALID_PLAYERS[@]}" 2>&1
