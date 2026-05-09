// Reference RPS bot in Go. Always plays PAPER.
// Wire protocol: judge/protocols/wire.md (stdio transport).
package main

import (
	"bufio"
	"fmt"
	"os"
	"strings"
)

func main() {
	r := bufio.NewScanner(os.Stdin)
	r.Buffer(make([]byte, 0, 1024), 64*1024)
	w := bufio.NewWriter(os.Stdout)
	defer w.Flush()

	for r.Scan() {
		tokens := strings.Fields(r.Text())
		if len(tokens) < 3 || tokens[0] != "EVENT" {
			continue
		}
		switch tokens[2] {
		case "GAME_STARTED", "ROUND_RESULT":
			fmt.Fprintln(w, "ACT MOVE PAPER")
			w.Flush()
		case "GAME_END":
			return
		}
	}
}
