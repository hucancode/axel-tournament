/* Heads-up no-limit Texas Hold'em starter skeleton in C.
 * Wire protocol: judge/protocols/wire.md. Spec: judge/protocols/poker.md. */
#include <stdio.h>
#include <string.h>

struct state {
    /* TODO: hand_no, dealer_idx, my_seat, hole, board, pot, last_action, ... */
    int _unused;
};

static const char *decide(struct state *s) {
    (void)s;
    /* TODO: return e.g. "FOLD" or "BET 50". */
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

        if (!strcmp(kind, "HAND_STARTED") || !strcmp(kind, "STREET") ||
            !strcmp(kind, "ACTION") || !strcmp(kind, "POT")) {
            const char *act = decide(&s);
            if (act) printf("ACT %s\n", act);
        } else if (!strcmp(kind, "GAME_END") || !strcmp(kind, "WINNER") ||
                   !strcmp(kind, "DRAW")) {
            return 0;
        }
    }
    return 0;
}
