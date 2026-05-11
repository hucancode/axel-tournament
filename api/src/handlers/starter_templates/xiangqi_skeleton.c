/* Xiangqi starter skeleton in C. Wire protocol: judge/protocols/wire.md.
 * Game spec: judge/protocols/xiangqi.md. */
#include <stdio.h>
#include <string.h>

struct state {
    int move_count;
    /* TODO: track board, side-to-move, my_seat, last_move, ... */
};

static const char *choose_move(struct state *s) {
    (void)s;
    /* TODO: return e.g. "b2 e2". */
    return NULL;
}

int main(void) {
    char line[4096];
    struct state s = {0};
    setvbuf(stdout, NULL, _IOLBF, 0);

    while (fgets(line, sizeof line, stdin)) {
        char ev[16], kind[32];
        long seq;
        if (sscanf(line, "%15s %ld %31s", ev, &seq, kind) < 3) continue;
        if (strcmp(ev, "EVENT") != 0) continue;

        if (!strcmp(kind, "GAME_STARTED")) {
            const char *mv = choose_move(&s);
            if (mv) printf("ACT MOVE %s\n", mv);
        } else if (!strcmp(kind, "MOVE")) {
            s.move_count++;
            const char *mv = choose_move(&s);
            if (mv) printf("ACT MOVE %s\n", mv);
        } else if (!strcmp(kind, "WINNER")) {
            return 0;
        }
    }
    return 0;
}
