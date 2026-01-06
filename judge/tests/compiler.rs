use std::path::Path;
use tempfile::TempDir;

#[cfg(test)]
mod compiler_tests {
    use std::fs;

    use super::*;
    use judge::sandbox::compiler::CompilerSandbox;

    #[tokio::test]
    async fn test_compiler_sandbox_creation() {
        let temp_dir = TempDir::new().unwrap();
        let result = CompilerSandbox::new(temp_dir.path().to_path_buf());
        assert!(result.is_ok(), "CompilerSandbox should be created");
        assert!(temp_dir.path().exists(), "Workspace should exist");
    }

    #[tokio::test]
    async fn test_compile_unsupported_language() {
        let temp_dir = TempDir::new().unwrap();
        let sandbox = CompilerSandbox::new(temp_dir.path().to_path_buf()).unwrap();

        let result = sandbox
            .compile("test_sub", "python", "print('hello')")
            .await;

        assert!(result.is_err(), "Should fail with unsupported language");
        if let Err(e) = result {
            assert!(e.to_string().contains("Unsupported language"));
        }
    }

    #[tokio::test]
    async fn test_compile_empty_code() {
        let temp_dir = TempDir::new().unwrap();
        let sandbox = CompilerSandbox::new(temp_dir.path().to_path_buf()).unwrap();

        let result = sandbox.compile("test_empty", "rust", "").await;

        // Should fail during compilation
        assert!(result.is_err(), "Empty code should fail to compile");
    }

    #[tokio::test]
    async fn test_compile_invalid_rust_code() {
        let temp_dir = TempDir::new().unwrap();
        let sandbox = CompilerSandbox::new(temp_dir.path().to_path_buf()).unwrap();

        let invalid_code = "fn main( { this is not valid rust }";
        let result = sandbox.compile("test_invalid", "rust", invalid_code).await;

        assert!(result.is_err(), "Invalid code should fail to compile");
        if let Err(e) = result {
            let error_msg = e.to_string();
            // The error could be compilation failure, cgroup setup failure (no permissions), or other
            // In any case, we just verify it fails as expected
            assert!(
                error_msg.contains("Compilation failed")
                    || error_msg.contains("Task join error")
                    || error_msg.contains("cgroup")
                    || error_msg.contains("failed"),
                "Expected error, got: {}",
                error_msg
            );
        }
    }

    #[tokio::test]
    async fn test_compile_valid_rust_code() {
        let temp_dir = TempDir::new().unwrap();
        let sandbox = CompilerSandbox::new(temp_dir.path().to_path_buf()).unwrap();

        let valid_code = r#"
            fn main() {
                println!("Hello, world!");
            }
        "#;

        let result = sandbox.compile("test_valid_rust", "rust", valid_code).await;
        assert!(
            result.is_ok(),
            "compilation of a valid code should be success"
        );
        let binary_path = result.unwrap();
        assert!(Path::new(&binary_path).exists(), "binary should exist");
        let metadata = fs::metadata(binary_path).unwrap();
        assert!(metadata.is_file(), "Should be a file");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = metadata.permissions().mode();
            assert!(mode & 0o111 != 0, "Binary should be executable");
        }
    }

    #[tokio::test]
    async fn test_compile_valid_c_code() {
        let temp_dir = TempDir::new().unwrap();
        let sandbox = CompilerSandbox::new(temp_dir.path().to_path_buf()).unwrap();

        let valid_code = r#"
            #include <stdio.h>
            int main() {
                printf("Hello from C\n");
                return 0;
            }
        "#;

        let result = sandbox.compile("test_valid_c", "c", valid_code).await;
        assert!(
            result.is_ok(),
            "compilation of a valid code should be success"
        );
        let binary_path = result.unwrap();
        assert!(Path::new(&binary_path).exists(), "binary should exist");
        let metadata = fs::metadata(binary_path).unwrap();
        assert!(metadata.is_file(), "Should be a file");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = metadata.permissions().mode();
            assert!(mode & 0o111 != 0, "Binary should be executable");
        }
    }

    #[tokio::test]
    async fn test_compile_valid_go_code() {
        let temp_dir = TempDir::new().unwrap();
        let sandbox = CompilerSandbox::new(temp_dir.path().to_path_buf()).unwrap();

        let valid_code = r#"
            package main
            import "fmt"
            func main() {
                fmt.Println("Hello from Go")
            }
        "#;

        let result = sandbox.compile("test_valid_go", "go", valid_code).await;
        assert!(
            result.is_ok(),
            "compilation of a valid code should be success"
        );
        let binary_path = result.unwrap();
        assert!(Path::new(&binary_path).exists(), "binary should exist");
        let metadata = fs::metadata(binary_path).unwrap();
        assert!(metadata.is_file(), "Should be a file");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = metadata.permissions().mode();
            assert!(mode & 0o111 != 0, "Binary should be executable");
        }
    }

    #[tokio::test]
    async fn test_workspace_isolation() {
        let temp_dir = TempDir::new().unwrap();
        let sandbox = CompilerSandbox::new(temp_dir.path().to_path_buf()).unwrap();
        let code = "fn main() {}";
        let _ = sandbox.compile("sub1", "rust", code).await;
        let _ = sandbox.compile("sub2", "rust", code).await;
        let ws1 = temp_dir.path().join("submission_sub1");
        let ws2 = temp_dir.path().join("submission_sub2");
        assert!(ws1.exists() && ws2.exists(), "Workspaces should be created");
    }
}
