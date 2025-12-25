use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LLMResponse {
    pub body: String,
    pub usage: Option<TokenUsage>,
}

/// Generic LLM Backend trait
#[async_trait]
pub trait LLMBackend: Send + Sync {
    async fn generate_function(
        &self,
        signature: &str,
        doc_comment: Option<&str>,
        context_snippets: Option<&[String]>,
        language: &str,
    ) -> Result<LLMResponse>;
}
