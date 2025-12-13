mod ai;
mod languages;
use ai::backend::LLMBackend;
use ai::gemini::GeminiBackend;
use ai::groq_gpt_oss::GGPTOSSBackend;
use languages::rust_backend;
use serde_json::json;
use std::env;
use std::io::{self, Read};

fn main() {
    let mut source_code = String::new();
    io::stdin()
        .read_to_string(&mut source_code)
        .expect("Failed to read source code from stdin");

    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Error: wrong args");
        println!("Usage: chace <cursor_byte> --backend <Gemini|Groq>");
        std::process::exit(1);
    }

    let cursor_byte: usize = args
        .last()
        .expect("Missing cursor_byte")
        .parse()
        .expect("Invalid cursor_byte");

    let backend_name = match args.iter().position(|a| a == "--backend") {
        Some(i) => args.get(i + 1).map(String::as_str),
        None => None,
    };

    let backend_name = backend_name.expect("Missing --backend");

    if let Some(func) =
        rust_backend::find_empty_function_at_cursor(&source_code, cursor_byte)
    {
        //let backend = GeminiBackend { api_key, model: "gemini-2.5-flash".to_string() };
        let backend: Box<dyn LLMBackend> = match backend_name {
            "Gemini" => {
                let api_key =
                    env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set");

                Box::new(GeminiBackend {
                    api_key,
                    model: "Gemini-2.5-flash".to_string(),
                })
            }
            "groq" => {
                let api_key =
                    env::var("GROQ_API_KEY").expect("GROQ_API_KEY not set");

                Box::new(GGPTOSSBackend { api_key, model: "openai/gpt-oss-20b".to_string() })
            }
            _ => {
                eprintln!("Unknown backend {}", backend_name);
                std::process::exit(1);
            }
        };

        let generated_body = backend
            .generate_function(
                &func.signature,
                func.doc_comment.as_deref(),
                "rust",
            )
            .unwrap();

        let output_json = json!({
            "start_byte": func.start_byte,
            "end_byte": func.end_byte,
            "body": generated_body,
        });

        println!("{}", output_json);
    } else {
        println!("No empty function");
    }
}
