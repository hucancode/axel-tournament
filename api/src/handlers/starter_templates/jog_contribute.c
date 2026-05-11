/* Reference Jar-of-Greed bot in C. Contributes a fixed amount each round.
 * Wire protocol: judge/protocols/wire.md (stdio transport).
 * Game spec: judge/protocols/jar-of-greed.md. */
#include <stdio.h>
#include <string.h>

#define CONTRIBUTION 1

int main(void) {
    char line[4096];
    setvbuf(stdout, NULL, _IOLBF, 0);
    while (fgets(line, sizeof line, stdin)) {
        char ev[16], kind[32];
        long seq;
        if (sscanf(line, "%15s %ld %31s", ev, &seq, kind) < 3) continue;
        if (strcmp(ev, "EVENT") != 0) continue;
        if (!strcmp(kind, "GAME_STARTED") || !strcmp(kind, "ROUND_RESULT")) {
            printf("ACT CONTRIBUTE %d\n", CONTRIBUTION);
        } else if (!strcmp(kind, "GAME_END")) {
            return 0;
        }
    }
    return 0;
}
