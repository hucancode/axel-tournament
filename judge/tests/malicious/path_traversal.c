// Malicious program attempting path traversal attacks
#include <stdio.h>
#include <stdlib.h>

int main() {
    // Try various path traversal techniques
    const char *paths[] = {
        "/../../../etc/passwd",
        "/./../../etc/passwd",
        "/player/../../../etc/passwd",
        "//etc/passwd",
        "/etc/../etc/passwd",
        NULL
    };

    int success_count = 0;
    for (int i = 0; paths[i] != NULL; i++) {
        FILE *f = fopen(paths[i], "r");
        if (f != NULL) {
            printf("SECURITY BREACH: Path traversal succeeded with: %s\n", paths[i]);
            fclose(f);
            success_count++;
        }
    }

    if (success_count == 0) {
        printf("BLOCKED: All path traversal attempts failed\n");
        return 1;
    }

    return 0;
}
