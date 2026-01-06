// Malicious program attempting to read /etc/shadow
#include <stdio.h>
#include <stdlib.h>

int main() {
    FILE *f = fopen("/etc/shadow", "r");
    if (f == NULL) {
        printf("BLOCKED: Cannot open /etc/shadow\n");
        return 1;
    }

    char buffer[1024];
    if (fgets(buffer, sizeof(buffer), f) != NULL) {
        printf("SECURITY BREACH: Read /etc/shadow: %s\n", buffer);
        fclose(f);
        return 0;
    }

    fclose(f);
    return 1;
}
