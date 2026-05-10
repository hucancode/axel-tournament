// CPU exhaustion. Spins forever; cgroup cpu.max + judge timeout must
// kill it within the configured budget. Test asserts process is reaped.
#include <stdio.h>
volatile long sink = 0;

int main(void) {
    printf("STARTED: tight cpu loop\n");
    fflush(stdout);
    for (;;) {
        sink ^= 0xdeadbeef;
        sink *= 31;
    }
    return 0;
}
