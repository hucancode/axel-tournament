// Reference Tic-Tac-Toe bot in Go. Tries cells left-to-right, top-to-bottom.
// Wire protocol: judge/protocols/wire.md (stdio transport).
//
// Bots don't know their seat (X or O). On every MOVE event we attempt
// the next cell in the preferred sequence. The TTT room logic silently
// ignores ACTs that aren't the bot's turn or hit an occupied cell, so
// off-turn attempts are harmless.
package main

import (
	"bufio"
	"fmt"
	"os"
	"strings"
)

func main() {
	cells := [][2]int{
		{0, 0}, {0, 1}, {0, 2},
		{1, 0}, {1, 1}, {1, 2},
		{2, 0}, {2, 1}, {2, 2},
	}
	r := bufio.NewScanner(os.Stdin)
	r.Buffer(make([]byte, 0, 1024), 64*1024)
	w := bufio.NewWriter(os.Stdout)
	defer w.Flush()

	idx := 0
	for r.Scan() {
		tokens := strings.Fields(r.Text())
		if len(tokens) < 3 || tokens[0] != "EVENT" {
			continue
		}
		switch tokens[2] {
		case "GAME_STARTED", "MOVE":
			if idx < len(cells) {
				fmt.Fprintf(w, "ACT MOVE %d %d\n", cells[idx][0], cells[idx][1])
				w.Flush()
				idx++
			}
		case "WINNER", "DRAW":
			return
		}
	}
}
