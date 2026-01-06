use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use crate::sandbox::compiler::CompilerSandbox;

pub struct Compiler {
    sandbox: Arc<CompilerSandbox>,
}

impl Compiler {
    pub fn new() -> Result<Self> {
        let workspace_root = std::env::var("COMPILER_WORKSPACE")
            .unwrap_or_else(|_| "/artifacts".to_string());

        let sandbox = Arc::new(
            CompilerSandbox::new(PathBuf::from(workspace_root))
                .map_err(|e| anyhow::anyhow!("Failed to create compiler sandbox: {}", e))?
        );

        Ok(Self { sandbox })
    }

    pub async fn compile_submission(
        &self,
        submission_id: &str,
        language: &str,
        code: &str,
    ) -> Result<String> {
        self.sandbox
            .compile(submission_id, language, code)
            .await
            .map_err(|e| anyhow::anyhow!("Compilation error: {}", e))
    }
}
