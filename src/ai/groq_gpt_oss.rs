use crate::ai::backend::LLMBackend;
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct GGPTOSSBackend {
    pub api_key: String,
    pub model: String,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct GROQRequest<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
}

#[derive(Deserialize)]
struct GGPTOSSResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageResponse,
}

#[derive(Deserialize)]
struct MessageResponse {
    content: String,
}

#[async_trait]
impl LLMBackend for GGPTOSSBackend {
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
        
        user_prompt.push_str(&format!("{}\n{} {{", doc_comment.unwrap_or(""), signature));

        let url = format!("https://api.groq.com/openai/v1/chat/completions",);

        let request_body = GROQRequest {
            model: &self.model,
            messages: vec![
                Message { role: "system", content: &system_prompt },
                Message { role: "user", content: &user_prompt },
            ],
        };

        let resp = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?
            .json::<GGPTOSSResponse>()
            .await?;

        let output = resp
           .choices
            .get(0)
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(Self::clean_output(&output))
    }
}

impl GGPTOSSBackend {
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
