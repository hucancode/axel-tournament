// execve("/bin/sh"). Gives an attacker an interactive shell on
// breakout. Sandbox should refuse exec of anything not whitelisted, or
// the shell binary should be unreachable via mount/landlock.
#include <stdio.h>
#include <unistd.h>
#include <errno.h>

int main(void) {
    char *argv[] = { "/bin/sh", "-c", "echo SECURITY BREACH: shell ran", NULL };
    execv("/bin/sh", argv);
    printf("BLOCKED: execv refused (errno=%d)\n", errno);
    return 1;
}
