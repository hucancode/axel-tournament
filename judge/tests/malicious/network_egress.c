// Net egress. Tries to open a TCP socket to an external host. Should be
// blocked by seccomp (no socket() in allowlist) or the network ns.
#include <stdio.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <unistd.h>
#include <string.h>

int main(void) {
    int s = socket(AF_INET, SOCK_STREAM, 0);
    if (s < 0) {
        printf("BLOCKED: socket() refused\n");
        return 1;
    }
    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_port = htons(80);
    addr.sin_addr.s_addr = inet_addr("8.8.8.8");

    if (connect(s, (struct sockaddr*)&addr, sizeof(addr)) < 0) {
        printf("BLOCKED: connect() refused\n");
        close(s);
        return 1;
    }
    printf("SECURITY BREACH: opened tcp to 8.8.8.8:80\n");
    close(s);
    return 0;
}
