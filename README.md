<div style="text-align: left;">
  <img src="logo/chace-logo.png" style="width:50%; margin: 0 auto;" />
</div>

*Pronounced: /tʃeɪs/ (chase)*

**CH**amal's **A**uto**C**omplete **E**ngine

## Overview

CHACE is a Rust-based engine designed for controlled AI-assisted code completion. Unlike traditional AI coding assistants that generate large code blocks, CHACE:

- lets you target empty function definitions at the cursor position
- Extracts the function signature and documentation (docstrings)
- Sends only the minimal context to the LLM
- Generates and inserts the function body

This approach keeps the AI focused on the specific task, reduces token usage, maintains precision and efficiency and produces more predictable results.

### Inspiration

CHACE is heavily inspired by ThePrimeagen's new approach to AI-assisted coding. While his implementation is built with Lua integrating Opencode (not yet open-sourced), CHACE takes a different architectural approach: built as a standalone Rust binary that operates independently and can be integrated into any editor through plugins. This design ensures CHACE is editor-agnostic, lightweight, and easy to extend to other development environments.

## Architecture

CHACE runs as a Unix socket server (`/tmp/chace.sock`) that accepts JSON requests containing source code and cursor position. The engine:

1. Parses the source code using Tree-sitter
2. Locates empty functions at the cursor
3. Sends function signatures to the configured LLM backend
4. Returns the generated function body with precise byte offsets

### Supported LLM Backends

- Google Gemini (gemini-2.5-flash)
- Groq (gpt-oss-20b)

### Language Support

Currently supports:
- Rust

## Installation

### Prerequisites

- Rust toolchain (2024 edition or later)
- API keys for LLM providers

### Build from Source

```bash
git clone https://github.com/yourusername/chace.git
cd chace
cargo build --release
```

### Configuration

Set the required environment variables:

```bash
export GEMINI_API_KEY="your-gemini-api-key"
export GROQ_API_KEY="your-groq-api-key"
```

## Usage

### Running the Server

```bash
./target/release/chace
```

The server listens on `/tmp/chace.sock` and handles concurrent connections.

### Request Format

Send JSON-encoded requests via the Unix socket:

```json
{
  "source_code": "fn add(a: i32, b: i32) -> i32 {\n\n}",
  "cursor_byte": 35,
  "backend": "Gemini"
}
```

### Response Format

```json
{
  "start_byte": 35,
  "end_byte": 36,
  "body": "    a + b",
  "error": null
}
```

### IDE Integration

CHACE is designed to be integrated with IDEs via plugins. See [chace.nvim](https://github.com/chamal1120/chace-nvim) for reference.

## Protocol

CHACE uses a line-delimited JSON protocol over Unix sockets:

- Each request is a single JSON object terminated by a newline
- Each response is a single JSON object terminated by a newline
- Multiple requests can be sent over the same connection
- Connections are handled asynchronously

## Development

### Adding Language Support

To add support for a new language:

1. Add the Tree-sitter grammar to `Cargo.toml`
2. Create a new backend in `src/languages/`
3. Implement function detection and signature extraction
4. Update the request handler in `main.rs`

### Adding LLM Backends

To add a new LLM provider:

1. Create a new module in `src/ai/`
2. Implement the `LLMBackend` trait
3. Add initialization in `main.rs`
4. Update the backend selection logic

## License

MIT License - see [LICENSE](LICENSE) for details.
