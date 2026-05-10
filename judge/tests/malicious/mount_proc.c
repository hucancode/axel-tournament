// Try mounting a fresh procfs to access host PIDs. Requires
// CAP_SYS_ADMIN, which the sandbox drops.
#include <stdio.h>
#include <sys/mount.h>
#include <errno.h>

int main(void) {
    if (mount("proc", "/tmp", "proc", 0, NULL) == 0) {
        printf("SECURITY BREACH: mounted procfs at /tmp\n");
        return 0;
    }
    printf("BLOCKED: mount refused (errno=%d)\n", errno);
    return 1;
}
