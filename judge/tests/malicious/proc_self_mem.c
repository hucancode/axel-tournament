// /proc/self/mem write attack. Some sandbox bypasses use it to rewrite
// .text. Landlock + procfs filter should refuse the open.
#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>

int main(void) {
    int fd = open("/proc/self/mem", O_RDWR);
    if (fd < 0) {
        printf("BLOCKED: /proc/self/mem open refused\n");
        return 1;
    }
    char buf[1] = { 0xCC };
    ssize_t n = pwrite(fd, buf, 1, 0x400000);
    if (n < 0) {
        printf("BLOCKED: write to /proc/self/mem refused\n");
        close(fd);
        return 1;
    }
    printf("SECURITY BREACH: wrote 1 byte at 0x400000\n");
    close(fd);
    return 0;
}
