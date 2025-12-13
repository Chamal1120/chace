use anyhow::Result;

/// Generic LLM Backend trait
pub trait LLMBackend {
    fn generate_function(
        &self,
        signature: &str,
        doc_comment: Option<&str>,
        language: &str,
    ) -> Result<String>;
}
