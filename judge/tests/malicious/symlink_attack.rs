// Malicious Rust program attempting symlink attacks
use std::fs;
use std::os::unix::fs as unix_fs;

fn main() {
    println!("Attempting symlink attacks...");

    // Try to create a symlink to /etc/passwd
    let symlink_result = unix_fs::symlink("/etc/passwd", "./passwd_link");
    match symlink_result {
        Ok(_) => {
            println!("Created symlink to /etc/passwd");

            // Try to read through the symlink
            match fs::read_to_string("./passwd_link") {
                Ok(contents) => {
                    println!("SECURITY BREACH: Read /etc/passwd via symlink");
                    println!("First line: {}", contents.lines().next().unwrap_or(""));
                    std::process::exit(0);
                }
                Err(e) => {
                    println!("Symlink created but read blocked: {}", e);
                }
            }
        }
        Err(e) => {
            println!("BLOCKED: Cannot create symlink: {}", e);
        }
    }

    // Try to traverse via symlink
    if let Ok(_) = unix_fs::symlink("../../..", "./escape") {
        match fs::read_dir("./escape/etc") {
            Ok(_) => {
                println!("SECURITY BREACH: Traversed via symlink to /etc");
                std::process::exit(0);
            }
            Err(e) => {
                println!("Symlink traversal blocked: {}", e);
            }
        }
    }

    println!("BLOCKED: All symlink attacks failed");
    std::process::exit(1);
}
