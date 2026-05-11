// Reference Jar-of-Greed bot in Go. Contributes a fixed amount each round.
// Wire protocol: judge/protocols/wire.md (stdio transport).
// Game spec: judge/protocols/jar-of-greed.md.
package main

import (
	"bufio"
	"fmt"
	"os"
	"strings"
)

const contribution = 1

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
			fmt.Fprintf(w, "ACT CONTRIBUTE %d\n", contribution)
			w.Flush()
		case "GAME_END":
			return
		}
	}
}
