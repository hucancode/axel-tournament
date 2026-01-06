#include <stdio.h>
#include <string.h>

int main() {
    char line[256];

    while (fgets(line, sizeof(line), stdin)) {
        // Remove newline
        line[strcspn(line, "\n")] = 0;

        if (strcmp(line, "START") == 0) {
            printf("ROCK\n");
            fflush(stdout);
        } else if (strncmp(line, "ROUND", 5) == 0) {
            printf("ROCK\n");
            fflush(stdout);
        } else if (strncmp(line, "SCORE", 5) == 0) {
            continue;
        } else if (strcmp(line, "END") == 0) {
            break;
        }
    }

    return 0;
}
