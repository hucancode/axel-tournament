// Memory exhaustion. Allocates 4 MiB chunks until OOM-killed by the
// memory cgroup or malloc returns NULL. Either is a pass.
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
    const size_t CHUNK = 4 * 1024 * 1024;
    size_t total = 0;
    for (int i = 0; i < 4096; i++) {
        char *p = malloc(CHUNK);
        if (!p) {
            printf("BLOCKED: malloc returned NULL after %zu MB\n", total / (1024 * 1024));
            return 1;
        }
        // Touch every page so the kernel really commits.
        memset(p, 0xAB, CHUNK);
        total += CHUNK;
    }
    printf("SECURITY BREACH: allocated %zu MB without limits\n", total / (1024 * 1024));
    return 0;
}
