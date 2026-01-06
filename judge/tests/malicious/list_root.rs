// Malicious Rust program attempting to list host root directory
use std::fs;

fn main() {
    match fs::read_dir("/") {
        Ok(entries) => {
            println!("PARTIAL BREACH: Can read / directory:");
            let mut found_suspicious = false;
            for entry in entries {
                if let Ok(entry) = entry {
                    let name = entry.file_name();
                    println!("  {:?}", name);

                    // If we see typical Linux directories, we're not properly isolated
                    let name_str = name.to_string_lossy();
                    if name_str == "etc" || name_str == "home" || name_str == "root" {
                        found_suspicious = true;
                    }
                }
            }

            if found_suspicious {
                println!("SECURITY BREACH: Host root directory visible!");
                std::process::exit(0);
            } else {
                println!("INFO: Root directory isolated (only saw player/usr)");
                std::process::exit(1);
            }
        }
        Err(e) => {
            println!("BLOCKED: Cannot list /: {}", e);
            std::process::exit(1);
        }
    }
}
