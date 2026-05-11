// Heads-up no-limit Texas Hold'em starter skeleton in Go.
// Wire protocol: judge/protocols/wire.md. Spec: judge/protocols/poker.md.
package main

import (
	"bufio"
	"fmt"
	"os"
	"strings"
)

type state struct {
	// TODO: hand_no, dealer_idx, my_seat, hole, board, pot, last_action, ...
}

func decide(s *state) string {
	// TODO: return e.g. "FOLD" or "BET 50".
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
		case "HAND_STARTED", "STREET", "ACTION", "POT":
			if act := decide(&s); act != "" {
				fmt.Fprintf(w, "ACT %s\n", act)
				w.Flush()
			}
		case "GAME_END", "WINNER", "DRAW":
			return
		}
	}
}
