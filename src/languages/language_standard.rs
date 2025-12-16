/// Information about a function extracted from the source code
#[derive(Debug)]
pub struct FunctionInfo {
    pub signature: String,
    pub doc_comment: Option<String>,
    pub start_byte: usize,
    pub end_byte: usize,
}

/// Trait for language-specific backend implementations
pub trait LanguageStandard {
    /// Finds an empty function at the cursor position and returns its information
    fn find_empty_function_at_cursor(
        &self,
        source_code: &str,
        cursor_byte: usize,
    ) -> Option<FunctionInfo>;

    /// Finds all empty functions in the source code
    fn find_empty_functions(&self, source_code: &str) -> Vec<FunctionInfo>;
}
