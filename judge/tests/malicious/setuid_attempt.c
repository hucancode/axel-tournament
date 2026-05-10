// Privilege escalation. With CAP_SETUID dropped (or seccomp denying
// setuid), this must fail.
#include <stdio.h>
#include <unistd.h>
#include <errno.h>

int main(void) {
    if (setuid(0) == 0 && geteuid() == 0) {
        printf("SECURITY BREACH: now root (euid=%d)\n", geteuid());
        return 0;
    }
    printf("BLOCKED: setuid(0) refused (errno=%d)\n", errno);
    return 1;
}
