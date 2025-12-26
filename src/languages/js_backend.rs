use crate::languages::helpers::{
    extract_doc_comment, extract_signature, is_empty_body, text_for,
};
use crate::languages::language_standard::{FunctionInfo, LanguageStandard};
use tree_sitter::Parser;
use tree_sitter_javascript;

pub struct JsBackend;

impl LanguageStandard for JsBackend {
    fn find_empty_function_at_cursor(
        &self,
        source_code: &str,
        cursor_byte: usize,
    ) -> Option<FunctionInfo> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_javascript::LANGUAGE.into())
            .expect("Failed to load Javascript grammar");

        let tree = parser.parse(source_code, None)?;
        let root = tree.root_node();
        let mut cursor = root.walk();

        for node in root.children(&mut cursor) {
            let kind = node.kind();
            if kind != "function_declaration" && kind != "method_definition" {
                continue;
            }

            // Check if the cursor is inside the function
            if cursor_byte < node.start_byte() || cursor_byte > node.end_byte()
            {
                continue;
            }

            // Check if empty
            let body = match node.child_by_field_name("body") {
                Some(body_node) => text_for(source_code, &body_node),
                None => continue,
            };
            if !is_empty_body(body) {
                continue;
            }

            let body_node = node.child_by_field_name("body").unwrap();

            let signature = extract_signature(&node, source_code);
            let doc_comment = extract_doc_comment(&node, source_code, "/**");

            return Some(FunctionInfo {
                signature,
                doc_comment,
                start_byte: body_node.start_byte() + 1,
                end_byte: body_node.end_byte() - 1,
            });
        }

        None
    }

    fn find_empty_functions(&self, source_code: &str) -> Vec<FunctionInfo> {
        let mut parser = Parser::new();

        parser
            .set_language(&tree_sitter_javascript::LANGUAGE.into())
            .expect("Error loading Javascript grammar");

        let tree = parser.parse(source_code, None).unwrap();
        let root_node = tree.root_node();
        let mut cursor = root_node.walk();
        let mut funcs = Vec::new();

        for node in root_node.children(&mut cursor) {
            if node.kind() != "function_item" {
                continue;
            }

            let body = match node.child_by_field_name("body") {
                Some(body_node) => text_for(source_code, &body_node),
                None => continue,
            };
            if !is_empty_body(body) {
                continue;
            }

            let signature = extract_signature(&node, source_code);
            let doc_comment = extract_doc_comment(&node, source_code, "/**");

            funcs.push(FunctionInfo {
                signature,
                doc_comment,
                start_byte: node.start_byte(),
                end_byte: node.end_byte(),
            });
        }

        funcs
    }
}

//-----------------------------Unit Tests--------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_simple_empty_function() {
        let backend = JsBackend;
        let code = r#"
/**
 * This is a test function
 */
function helloWorld(name) {
}"#;
        let cursor_byte = code.find('}').unwrap() - 1;
        
        let result = backend.find_empty_function_at_cursor(code, cursor_byte);
        
        assert!(result.is_some(), "Should find the empty function");
        let info = result.unwrap();
        
        assert_eq!(info.signature, "function helloWorld(name)");
        assert_eq!(info.doc_comment, Some("This is a test function".to_string()));
        
        let body_content = &code[info.start_byte..info.end_byte];
        assert!(body_content.trim().is_empty());
    }

    #[test]
    fn test_find_async_function() {
        let backend = JsBackend;
        let code = r#"
/**
 * Multi-line doc
 * second line
 */
async function fetchData(url) {
}"#;
        let cursor_byte = code.find('{').unwrap() + 1;
        let result = backend.find_empty_function_at_cursor(code, cursor_byte).unwrap();
        
        assert!(result.signature.contains("async function fetchData"));
        assert_eq!(result.doc_comment, Some("Multi-line doc\nsecond line".to_string()));
    }

    #[test]
    fn test_ignores_populated_function() {
        let backend = JsBackend;
        let code = r#"function hasCode() { console.log("hi"); }"#;
        let cursor_byte = code.find('c').unwrap();
        
        let result = backend.find_empty_function_at_cursor(code, cursor_byte);
        assert!(result.is_none(), "Should ignore functions with bodies");
    }

    #[test]
    fn test_cursor_outside_function() {
        let backend = JsBackend;
        let code = "function empty() {} \n // cursor is here";
        let cursor_byte = code.len() - 1;
        
        let result = backend.find_empty_function_at_cursor(code, cursor_byte);
        assert!(result.is_none(), "Should ignore if cursor is outside bounds");
    }
}
