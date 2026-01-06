package main

import (
	"bufio"
	"fmt"
	"os"
	"strings"
)

func main() {
	scanner := bufio.NewScanner(os.Stdin)

	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())

		if line == "START" {
			fmt.Println("ROCK")
		} else if strings.HasPrefix(line, "ROUND") {
			fmt.Println("ROCK")
		} else if strings.HasPrefix(line, "SCORE") {
			continue
		} else if line == "END" {
			break
		}
	}
}
