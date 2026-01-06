// Malicious Rust program attempting to access inherited file descriptors
use std::fs::File;
use std::io::Read;
use std::os::unix::io::FromRawFd;

fn main() {
    println!("Attempting to access file descriptors 3-20...");

    let mut breaches = 0;
    for fd in 3..=20 {
        unsafe {
            // Try to create a File from raw fd
            let result = std::panic::catch_unwind(|| {
                let mut file = File::from_raw_fd(fd);
                let mut buffer = [0u8; 64];
                if let Ok(n) = file.read(&mut buffer) {
                    if n > 0 {
                        return Some(n);
                    }
                }
                None
            });

            if let Ok(Some(bytes)) = result {
                println!("SECURITY BREACH: FD {} is readable ({} bytes)", fd, bytes);
                breaches += 1;
            }
        }
    }

    if breaches > 0 {
        println!("SECURITY BREACH: {} file descriptors were accessible", breaches);
        std::process::exit(0);
    } else {
        println!("BLOCKED: No leaked file descriptors");
        std::process::exit(1);
    }
}
