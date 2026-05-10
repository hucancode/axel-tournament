// Ptrace attempt. Even attaching to your own child counts — seccomp
// allowlist must not include ptrace().
#include <stdio.h>
#include <sys/ptrace.h>
#include <errno.h>

int main(void) {
    long r = ptrace(PTRACE_TRACEME, 0, 0, 0);
    if (r < 0) {
        printf("BLOCKED: ptrace refused (errno=%d)\n", errno);
        return 1;
    }
    printf("SECURITY BREACH: ptrace() succeeded\n");
    return 0;
}
