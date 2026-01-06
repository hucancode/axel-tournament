// Malicious program attempting to write to /tmp
#include <stdio.h>
#include <stdlib.h>

int main() {
    FILE *f = fopen("/tmp/malicious_file", "w");
    if (f == NULL) {
        printf("BLOCKED: Cannot write to /tmp\n");
        return 1;
    }

    if (fprintf(f, "malicious data") > 0) {
        printf("SECURITY BREACH: Wrote to /tmp/malicious_file\n");
        fclose(f);
        return 0;
    }

    fclose(f);
    return 1;
}
