/* Reference RPS bot in C. Always plays PAPER.
 * Wire protocol: judge/protocols/wire.md (stdio transport). */
#include <stdio.h>
#include <string.h>

int main(void) {
    char line[256];
    setvbuf(stdout, NULL, _IOLBF, 0);
    while (fgets(line, sizeof line, stdin)) {
        char verb[16] = {0}, kind[32] = {0};
        unsigned long seq;
        if (sscanf(line, "%15s %lu %31s", verb, &seq, kind) < 3) continue;
        if (strcmp(verb, "EVENT") != 0) continue;
        if (strcmp(kind, "GAME_STARTED") == 0 || strcmp(kind, "ROUND_RESULT") == 0) {
            puts("ACT MOVE PAPER");
            fflush(stdout);
        } else if (strcmp(kind, "GAME_END") == 0) {
            break;
        }
    }
    return 0;
}
