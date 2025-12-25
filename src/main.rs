mod ai;
mod languages;
use ai::backend::{ LLMBackend, TokenUsage };
use ai::gemini::GeminiBackend;
use ai::groq_gpt_oss::GGPTOSSBackend;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, split};
use tokio::net::{UnixListener, UnixStream};

#[derive(Deserialize)]
struct GenerateRequest {
    source_code: String,
    cursor_byte: usize,
    backend: String,
    file_type: String, 
    #[serde(default)]
    context_snippets: Option<Vec<String>>,
}

#[derive(Serialize)]
struct GenerateResponse {
    start_byte: usize,
    end_byte: usize,
    body: String,
    usage: Option<TokenUsage>,
    error: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize backends
    let gemini = Arc::new(GeminiBackend {
        api_key: std::env::var("GEMINI_API_KEY")?,
        model: "Gemini-2.5-flash".to_string(),
    });

    let groq = Arc::new(GGPTOSSBackend {
        api_key: std::env::var("GROQ_API_KEY")?,
        model: "openai/gpt-oss-20b".to_string(),
    });

    let path = "/tmp/chace.sock";
    if Path::new(path).exists() {
        std::fs::remove_file(path)?;
    }

    let listener = UnixListener::bind(path)?;

    loop {
        let (socket, _) = listener.accept().await?;
        let gemini = Arc::clone(&gemini);
        let groq = Arc::clone(&groq);

        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, gemini, groq).await {
                eprintln!("connection error: {e}");
            }
        });
    }
}

async fn handle_connection(
    socket: UnixStream,
    gemini: Arc<GeminiBackend>,
    groq: Arc<GGPTOSSBackend>,
) -> anyhow::Result<()> {
    let (reader, mut writer) = split(socket);
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    while reader.read_line(&mut line).await? != 0 {
        let req: GenerateRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                writer
                    .write_all(format!("{{\"error\":\"{e}\"}}\n").as_bytes())
                    .await?;
                line.clear();
                continue;
            }
        };

        let resp = handle_request(req, &gemini, &groq).await;
        let json = serde_json::to_string(&resp)?;
        writer.write_all(json.as_bytes()).await?;
        writer.write_all(b"\n").await?;

        line.clear();
    }

    Ok(())
}

async fn handle_request(
    req: GenerateRequest,
    gemini: &Arc<GeminiBackend>,
    groq: &Arc<GGPTOSSBackend>,
) -> GenerateResponse {
    use languages::rust_backend::RustBackend;
    use languages::ts_backend::TsBackend;
    use languages::language_standard::LanguageStandard;

    let backend_opt: Option<Box<dyn LanguageStandard + Send>> = match req.file_type.as_str() {
        "rust" => Some(Box::new(RustBackend)),
        "ts"| "typescript" => Some(Box::new(TsBackend)),
        _ => None,
    };

    let Some(backend) = backend_opt else {
        return GenerateResponse {
            start_byte: 0,
            end_byte: 0,
            body: String::new(),
            usage: None,
            error: Some("Unsupported language".into()),
        };
    };

    let Some(func) = backend.find_empty_function_at_cursor(
        &req.source_code,
        req.cursor_byte,
    ) else {
        return GenerateResponse {
            start_byte: 0,
            end_byte: 0,
            body: String::new(),
            usage: None,
            error: Some("No empty function".into()),
        };
    };

    let backend: &dyn LLMBackend = match req.backend.as_str() {
        "Gemini" => gemini.as_ref(),
        "groq" => groq.as_ref(),
        _ => {
            return GenerateResponse {
                start_byte: 0,
                end_byte: 0,
                body: String::new(),
                usage: None,
                error: Some("Unknown backend".into()),
            };
        }
    };

    match backend
        .generate_function(
            &func.signature,
            func.doc_comment.as_deref(),
            req.context_snippets.as_deref(),
            req.file_type.as_ref(),
        )
        .await
    {
        Ok(res) => GenerateResponse {
            start_byte: func.start_byte,
            end_byte: func.end_byte,
            body: res.body,
            usage: res.usage,
            error: None,
        },
        Err(e) => GenerateResponse {
            start_byte: func.start_byte,
            end_byte: func.end_byte,
            body: String::new(),
            usage: None,
            error: Some(e.to_string()),
        },
    }
}
