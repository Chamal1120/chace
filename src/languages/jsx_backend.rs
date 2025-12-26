use crate::languages::helpers::{
    extract_doc_comment, extract_signature, is_empty_body, text_for,
};
use crate::languages::language_standard::{FunctionInfo, LanguageStandard};
use tree_sitter::Parser;
use tree_sitter_javascript;

pub struct JsxBackend;

impl LanguageStandard for JsxBackend {
    fn find_empty_function_at_cursor(
        &self,
        source_code: &str,
        cursor_byte: usize,
    ) -> Option<FunctionInfo> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_javascript::LANGUAGE.into())
            .expect("Failed to load Typescript grammar");

        let tree = parser.parse(source_code, None)?;
        let root = tree.root_node();
        let mut cursor = root.walk();

        for node in root.children(&mut cursor) {
            let kind = node.kind();

            // Check bounds first to save processing
            if cursor_byte < node.start_byte() || cursor_byte > node.end_byte()
            {
                continue;
            }
            // Identify the "real" function node
            let target_node = match kind {
                "function_declaration" | "method_definition" => Some(node),
                "export_statement" | "lexical_declaration" => {
                    find_arrow_recursive(node)
                }
                _ => None,
            };

            if let Some(func_node) = target_node {
                if let Some(body_node) = func_node.child_by_field_name("body") {
                    let body_text = text_for(source_code, &body_node);

                    if is_empty_body(body_text) {
                        return Some(FunctionInfo {
                            signature: extract_signature(&node, source_code),
                            doc_comment: extract_doc_comment(
                                &node,
                                source_code,
                                "/**",
                            ),
                            start_byte: body_node.start_byte() + 1,
                            end_byte: body_node.end_byte() - 1,
                        });
                    }
                }
            }
        }

        None
    }

    fn find_empty_functions(&self, source_code: &str) -> Vec<FunctionInfo> {
        let mut parser = Parser::new();

        parser
            .set_language(&tree_sitter_javascript::LANGUAGE.into())
            .expect("Error loading Javascriptreact grammar");

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

//---------------------- Backend specific helpers -----------------------------

fn find_arrow_recursive(node: tree_sitter::Node) -> Option<tree_sitter::Node> {
    if node.kind() == "arrow_function" {
        return Some(node);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(found) = find_arrow_recursive(child) {
            return Some(found);
        }
    }
    None
}

//-----------------------------Unit Tests--------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_simple_empty_function() {
        let backend = JsxBackend;
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
    fn test_find_arrow_function() {
        let backend = JsxBackend;
        let code = r#"
/**
 * Arrow function test
 */
export const myFunc = (x) => {
}"#;
        let cursor_byte = code.find('}').unwrap() - 1;
        
        let result = backend.find_empty_function_at_cursor(code, cursor_byte);
        
        assert!(result.is_some(), "Should find the empty arrow function");
        let info = result.unwrap();
        
        assert!(info.signature.contains("export const myFunc"));
        assert_eq!(info.doc_comment, Some("Arrow function test".to_string()));
    }

    #[test]
    fn test_ignores_populated_function() {
        let backend = JsxBackend;
        let code = r#"function hasCode() { console.log("hi"); }"#;
        let cursor_byte = code.find('c').unwrap();
        
        let result = backend.find_empty_function_at_cursor(code, cursor_byte);
        assert!(result.is_none(), "Should ignore functions with bodies");
    }

    #[test]
    fn test_cursor_outside_function() {
        let backend = JsxBackend;
        let code = "function empty() {} \n // cursor is here";
        let cursor_byte = code.len() - 1;
        
        let result = backend.find_empty_function_at_cursor(code, cursor_byte);
        assert!(result.is_none(), "Should ignore if cursor is outside bounds");
    }
}
