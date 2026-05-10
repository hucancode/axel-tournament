// Fork bomb. Tries to spawn ~256 children to exhaust pids.max in the
// cgroup. Sandbox should kill or refuse fork() before that happens.
#include <stdio.h>
#include <unistd.h>

int main(void) {
    int forks = 0;
    for (int i = 0; i < 256; i++) {
        pid_t p = fork();
        if (p < 0) {
            // fork() refused -> sandbox limit hit, this is the win path.
            printf("BLOCKED at fork %d\n", i);
            return 1;
        }
        if (p == 0) {
            // Child loops forever to keep its pid slot busy.
            while (1) sleep(1);
        }
        forks++;
    }
    printf("SECURITY BREACH: forked %d processes\n", forks);
    return 0;
}
