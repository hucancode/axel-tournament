// Malicious Rust program attempting to read /etc/passwd
use std::fs;

fn main() {
    match fs::read_to_string("/etc/passwd") {
        Ok(contents) => {
            println!("SECURITY BREACH: Read /etc/passwd");
            println!("First line: {}", contents.lines().next().unwrap_or(""));
            std::process::exit(0);
        }
        Err(e) => {
            println!("BLOCKED: Cannot read /etc/passwd: {}", e);
            std::process::exit(1);
        }
    }
}
