use crate::ai::backend::LLMBackend;
use crate::ai::helpers::clean_output;
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
        context_snippets: Option<&[String]>,
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

        let mut user_prompt = String::new();

        if let Some(snippets) = context_snippets {
            if !snippets.is_empty() {
                user_prompt.push_str("Context code for reference:\n");
                for snippet in snippets {
                    user_prompt.push_str("---\n");
                    user_prompt.push_str(snippet);
                    user_prompt.push_str("\n---\n\n");
                }
            }
        }

        user_prompt.push_str(&format!(
            "{}\n{} {{",
            doc_comment.unwrap_or(""),
            signature
        ));

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

        Ok(clean_output(&output))
    }
}
