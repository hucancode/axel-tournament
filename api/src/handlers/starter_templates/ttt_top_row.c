/* Reference Tic-Tac-Toe bot in C. Tries cells left-to-right, top-to-bottom.
 * Wire protocol: judge/protocols/wire.md (stdio transport).
 *
 * Bots don't know their seat. On every MOVE event we attempt the next
 * cell in the preferred sequence. The TTT room logic silently ignores
 * ACTs that aren't the bot's turn or hit an occupied cell. */
#include <stdio.h>
#include <string.h>

int main(void) {
    int cells[9][2] = {
        {0,0},{0,1},{0,2},
        {1,0},{1,1},{1,2},
        {2,0},{2,1},{2,2},
    };
    int idx = 0;
    char line[4096];
    setvbuf(stdout, NULL, _IOLBF, 0);
    while (fgets(line, sizeof line, stdin)) {
        char ev[16], kind[32];
        long seq;
        if (sscanf(line, "%15s %ld %31s", ev, &seq, kind) < 3) continue;
        if (strcmp(ev, "EVENT") != 0) continue;
        if (!strcmp(kind, "GAME_STARTED") || !strcmp(kind, "MOVE")) {
            if (idx < 9) {
                printf("ACT MOVE %d %d\n", cells[idx][0], cells[idx][1]);
                idx++;
            }
        } else if (!strcmp(kind, "WINNER") || !strcmp(kind, "DRAW")) {
            return 0;
        }
    }
    return 0;
}
