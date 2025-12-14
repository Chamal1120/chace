use anyhow::Result;
use async_trait::async_trait;

/// Generic LLM Backend trait
#[async_trait]
pub trait LLMBackend: Send + Sync {
    async fn generate_function(
        &self,
        signature: &str,
        doc_comment: Option<&str>,
        context_snippets: Option<&[String]>,
        language: &str,
    ) -> Result<String>;
}
