// Malicious program attempting to read /etc/passwd
#include <stdio.h>
#include <stdlib.h>

int main() {
    FILE *f = fopen("/etc/passwd", "r");
    if (f == NULL) {
        printf("BLOCKED: Cannot open /etc/passwd\n");
        return 1;
    }

    char buffer[1024];
    if (fgets(buffer, sizeof(buffer), f) != NULL) {
        printf("SECURITY BREACH: Read /etc/passwd: %s\n", buffer);
        fclose(f);
        return 0;
    }

    fclose(f);
    return 1;
}
