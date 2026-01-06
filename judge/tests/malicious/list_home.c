// Malicious program attempting to list /home directory
#include <stdio.h>
#include <dirent.h>
#include <errno.h>
#include <string.h>

int main() {
    DIR *dir = opendir("/home");
    if (dir == NULL) {
        printf("BLOCKED: Cannot open /home: %s\n", strerror(errno));
        return 1;
    }

    struct dirent *entry;
    printf("SECURITY BREACH: /home directory contents:\n");
    while ((entry = readdir(dir)) != NULL) {
        printf("  %s\n", entry->d_name);
    }

    closedir(dir);
    return 0;
}
