// Xiangqi starter skeleton in Go. Wire protocol: judge/protocols/wire.md.
// Game spec: judge/protocols/xiangqi.md.
package main

import (
	"bufio"
	"fmt"
	"os"
	"strings"
)

type state struct {
	moveCount int
	// TODO: track board, side-to-move, my_seat, last_move, ...
}

func chooseMove(s *state) string {
	// TODO: return e.g. "b2 e2" to advance the red cannon.
	return ""
}

func main() {
	r := bufio.NewScanner(os.Stdin)
	r.Buffer(make([]byte, 0, 1024), 64*1024)
	w := bufio.NewWriter(os.Stdout)
	defer w.Flush()
	var s state

	for r.Scan() {
		tokens := strings.Fields(r.Text())
		if len(tokens) < 3 || tokens[0] != "EVENT" {
			continue
		}
		switch tokens[2] {
		case "GAME_STARTED":
			if mv := chooseMove(&s); mv != "" {
				fmt.Fprintf(w, "ACT MOVE %s\n", mv)
				w.Flush()
			}
		case "MOVE":
			s.moveCount++
			if mv := chooseMove(&s); mv != "" {
				fmt.Fprintf(w, "ACT MOVE %s\n", mv)
				w.Flush()
			}
		case "WINNER":
			return
		}
	}
}
