// Malicious program attempting to read /proc/self/environ
#include <stdio.h>
#include <stdlib.h>

int main() {
    FILE *f = fopen("/proc/self/environ", "r");
    if (f == NULL) {
        printf("BLOCKED: Cannot open /proc/self/environ\n");
        return 1;
    }

    char buffer[4096];
    size_t bytes = fread(buffer, 1, sizeof(buffer) - 1, f);
    if (bytes > 0) {
        printf("SECURITY BREACH: Read /proc/self/environ (%zu bytes)\n", bytes);
        fclose(f);
        return 0;
    }

    fclose(f);
    return 1;
}
