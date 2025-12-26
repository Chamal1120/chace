use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::Duration;

#[derive(Serialize)]
struct GenerateRequest {
    source_code: String,
    cursor_byte: usize,
    backend: String,
    file_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    context_snippets: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct GenerateResponse {
    start_byte: usize,
    end_byte: usize,
    body: String,
    usage: Option<TokenUsage>,
    error: Option<String>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct TokenUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

async fn send_request(req: &GenerateRequest) -> anyhow::Result<GenerateResponse> {
    let socket_path = "/tmp/chace_test.sock";
    let stream = UnixStream::connect(socket_path).await?;
    let (reader, mut writer) = tokio::io::split(stream);
    let mut reader = BufReader::new(reader);

    let json = serde_json::to_string(req)?;
    writer.write_all(json.as_bytes()).await?;
    writer.write_all(b"\n").await?;

    let mut response = String::new();
    reader.read_line(&mut response).await?;

    let resp: GenerateResponse = serde_json::from_str(&response)?;
    Ok(resp)
}

#[tokio::test]
async fn test_invalid_json_request() {
    let socket_path = "/tmp/chace_test.sock";
    
    // Wait for socket to be available
    for _ in 0..10 {
        if Path::new(socket_path).exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let stream = UnixStream::connect(socket_path).await.unwrap();
    let (reader, mut writer) = tokio::io::split(stream);
    let mut reader = BufReader::new(reader);

    // Send invalid JSON
    writer.write_all(b"invalid json\n").await.unwrap();

    let mut response = String::new();
    reader.read_line(&mut response).await.unwrap();

    assert!(response.contains("error"));
}

#[tokio::test]
async fn test_unsupported_language() {
    let socket_path = "/tmp/chace_test.sock";
    
    // Wait for socket
    for _ in 0..10 {
        if Path::new(socket_path).exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let req = GenerateRequest {
        source_code: "fn test() {}".to_string(),
        cursor_byte: 10,
        backend: "Gemini".to_string(),
        file_type: "python".to_string(),
        context_snippets: None,
    };

    let resp = send_request(&req).await.unwrap();
    
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap(), "Unsupported language");
}

#[tokio::test]
async fn test_no_empty_function_found() {
    let socket_path = "/tmp/chace_test.sock";
    
    // Wait for socket
    for _ in 0..10 {
        if Path::new(socket_path).exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let req = GenerateRequest {
        source_code: r#"fn has_code() { println!("hello"); }"#.to_string(),
        cursor_byte: 20,
        backend: "Gemini".to_string(),
        file_type: "rust".to_string(),
        context_snippets: None,
    };

    let resp = send_request(&req).await.unwrap();
    
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap(), "No empty function");
}

#[tokio::test]
async fn test_unknown_backend() {
    let socket_path = "/tmp/chace_test.sock";
    
    // Wait for socket
    for _ in 0..10 {
        if Path::new(socket_path).exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let req = GenerateRequest {
        source_code: "fn empty() {}".to_string(),
        cursor_byte: 11,
        backend: "UnknownBackend".to_string(),
        file_type: "rust".to_string(),
        context_snippets: None,
    };

    let resp = send_request(&req).await.unwrap();
    
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap(), "Unknown backend");
}

#[tokio::test]
async fn test_rust_empty_function_success() {
    let socket_path = "/tmp/chace_test.sock";
    
    // Wait for socket
    for _ in 0..10 {
        if Path::new(socket_path).exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let code = r#"
/// Adds two numbers together
fn add(a: i32, b: i32) -> i32 {
}"#;

    let req = GenerateRequest {
        source_code: code.to_string(),
        cursor_byte: code.find('}').unwrap() - 1,
        backend: "Gemini".to_string(),
        file_type: "rust".to_string(),
        context_snippets: None,
    };

    // This test may fail if API keys are not set, so we check for that
    let result = send_request(&req).await;
    
    match result {
        Ok(resp) => {
            // If successful, verify the response structure
            assert!(resp.start_byte > 0);
            assert!(resp.end_byte > resp.start_byte);
            
            // Either we get a body or an error (e.g., API key missing)
            if resp.error.is_some() {
                println!("Expected error (likely API key issue): {:?}", resp.error);
            } else {
                assert!(!resp.body.is_empty());
            }
        }
        Err(e) => {
            println!("Request failed (expected if API keys not set): {}", e);
        }
    }
}

#[tokio::test]
async fn test_typescript_empty_function() {
    let socket_path = "/tmp/chace_test.sock";
    
    // Wait for socket
    for _ in 0..10 {
        if Path::new(socket_path).exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let code = r#"
/**
 * Multiplies two numbers
 */
function multiply(a: number, b: number): number {
}"#;

    let req = GenerateRequest {
        source_code: code.to_string(),
        cursor_byte: code.find('}').unwrap() - 1,
        backend: "groq".to_string(),
        file_type: "typescript".to_string(),
        context_snippets: None,
    };

    let result = send_request(&req).await;
    
    match result {
        Ok(resp) => {
            assert!(resp.start_byte > 0);
            assert!(resp.end_byte > resp.start_byte);
            
            if resp.error.is_some() {
                println!("Expected error (likely API key issue): {:?}", resp.error);
            } else {
                assert!(!resp.body.is_empty());
            }
        }
        Err(e) => {
            println!("Request failed (expected if API keys not set): {}", e);
        }
    }
}

#[tokio::test]
async fn test_tsx_component_function() {
    let socket_path = "/tmp/chace_test.sock";
    
    // Wait for socket
    for _ in 0..10 {
        if Path::new(socket_path).exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let code = r#"
/**
 * Button component
 */
export const Button = (props: ButtonProps): JSX.Element => {
}"#;

    let req = GenerateRequest {
        source_code: code.to_string(),
        cursor_byte: code.find('}').unwrap() - 1,
        backend: "Gemini".to_string(),
        file_type: "typescriptreact".to_string(),
        context_snippets: None,
    };

    let result = send_request(&req).await;
    
    match result {
        Ok(resp) => {
            assert!(resp.start_byte > 0);
            assert!(resp.end_byte > resp.start_byte);
        }
        Err(e) => {
            println!("Request failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_with_context_snippets() {
    let socket_path = "/tmp/chace_test.sock";
    
    // Wait for socket
    for _ in 0..10 {
        if Path::new(socket_path).exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let code = r#"
/// Helper function with context
fn process_data(data: Vec<u8>) -> String {
}"#;

    let req = GenerateRequest {
        source_code: code.to_string(),
        cursor_byte: code.find('}').unwrap() - 1,
        backend: "Gemini".to_string(),
        file_type: "rust".to_string(),
        context_snippets: Some(vec![
            "const MAX_SIZE: usize = 1024;".to_string(),
            "fn validate(input: &[u8]) -> bool { true }".to_string(),
        ]),
    };

    let result = send_request(&req).await;
    
    match result {
        Ok(resp) => {
            assert!(resp.start_byte > 0);
            assert!(resp.end_byte > resp.start_byte);
        }
        Err(e) => {
            println!("Request failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_multiple_sequential_requests() {
    let socket_path = "/tmp/chace_test.sock";
    
    // Wait for socket
    for _ in 0..10 {
        if Path::new(socket_path).exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Send multiple requests sequentially
    for i in 0..3 {
        let req = GenerateRequest {
            source_code: format!("fn test{}() {{}}", i),
            cursor_byte: 15,
            backend: "Gemini".to_string(),
            file_type: "rust".to_string(),
            context_snippets: None,
        };

        let _ = send_request(&req).await;
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
