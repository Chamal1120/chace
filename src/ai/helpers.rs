/// Cleanup of markdown/code fences, extra text
pub fn clean_output(output: &str) -> String {
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
