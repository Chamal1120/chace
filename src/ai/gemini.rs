use crate::ai::backend::LLMBackend;
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Gemini backend implementation
pub struct GeminiBackend {
    pub api_key: String,
    pub model: String,
}

// Request body
#[derive(Serialize)]
struct GeminiRequest<'a> {
    contents: Vec<GeminiContent<'a>>,
}
#[derive(Serialize)]
struct GeminiContent<'a> {
    role: &'a str,
    parts: Vec<GeminiPart<'a>>,
}
#[derive(Serialize)]
struct GeminiPart<'a> {
    text: &'a str,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}
#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiCandidateContent,
}
#[derive(Deserialize)]
struct GeminiCandidateContent {
    parts: Vec<GeminiPartResponse>,
}
#[derive(Deserialize)]
struct GeminiPartResponse {
    text: String,
}

#[async_trait]
impl LLMBackend for GeminiBackend {
    async fn generate_function(
        &self,
        signature: &str,
        doc_comment: Option<&str>,
        language: &str,
    ) -> Result<String> {
        let client = Client::new();

        let system_prompt = format!(
            "You are an AI {} code generator.\n\
             Complete only the body of the function.\n\
             Do NOT add explanations or markdown.\n\
             Respond only with valid {} code inside the braces.",
            language, language
        );

        let user_prompt =
            format!("{}\n{} {{", doc_comment.unwrap_or(""), signature);

        let full_prompt = format!("{}\n{}", system_prompt, user_prompt);

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            self.model
        );

        let request_body = GeminiRequest {
            contents: vec![GeminiContent {
                role: "user",
                parts: vec![GeminiPart { text: &full_prompt }],
            }],
        };

        let resp = client
            .post(&url)
            .header("x-goog-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?
            .json::<GeminiResponse>()
            .await?;

        let output = resp
            .candidates
            .get(0)
            .and_then(|c| c.content.parts.get(0))
            .map(|p| p.text.clone())
            .unwrap_or_default();

        Ok(Self::clean_output(&output))
    }
}

impl GeminiBackend {
    /// Cleanup of markdown/code fences, extra text
    fn clean_output(output: &str) -> String {
        let mut out = output.trim();

        // Remove fences like ```rust
        if out.starts_with("```") {
            out = out.trim_start_matches("```");
            if let Some(idx) = out.find('\n') {
                out = &out[idx..];
            }
        }

        // Remove ending ```
        if out.ends_with("```") {
            out = out.trim_end_matches("```");
        }

        out.trim().to_string()
    }
}
