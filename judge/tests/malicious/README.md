# Malicious Programs

This directory contains malicious programs designed to test the sandbox's security mechanisms (landlock and rootfs isolation).
The following programs attempt various escape techniques:

1. **read_passwd.c** - Attempts to read `/etc/passwd`
2. **read_shadow.c** - Attempts to read `/etc/shadow`
3. **list_home.c** - Attempts to list the `/home` directory
4. **read_proc_self.c** - Attempts to read `/proc/self/environ`
5. **path_traversal.c** - Attempts path traversal attacks (`../../../etc/passwd`, etc.)
6. **write_tmp.c** - Attempts to write files to `/tmp`

## Expected Behavior

When run in the sandbox, ALL of these programs should:
- Print "BLOCKED: Cannot ..." messages
- Exit with non-zero exit code (failure)
- **NOT** print any "SECURITY BREACH" messages

The sandbox uses multiple layers of defense:
1. **pivot_root** - Isolates the filesystem view so `/etc`, `/home`, etc. don't exist
2. **landlock** - Kernel-enforced path access control restricting file operations
3. **namespaces** - Process, mount, and network isolation
4. **cgroups** - Resource limiting
5. **seccomp** - System call filtering
6. **capabilities** - Dropped to prevent privilege escalation

## Running the Tests

### Automated Tests

The security tests are in `tests/pentest.rs`. They require root/CAP_SYS_ADMIN privileges:

```bash
sudo cargo test --test security_tests -- --ignored --nocapture
```

### Manual Testing

To manually compile and run a malicious program:

```bash
# 1. Compile the malicious program with gcc
cd tests/malicious
gcc -o read_passwd read_passwd.c

# 2. Run it WITHOUT sandbox (should succeed on most systems)
./read_passwd
# Expected output: Shows first line of /etc/passwd

# 3. Run through the judge's sandbox (requires running the judge)
# This would need to be done through the judge's API/compilation pipeline
# The program should be BLOCKED
```
