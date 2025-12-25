use crate::languages::helpers::{
    extract_doc_comment, extract_signature, is_empty_body, text_for,
};
use crate::languages::language_standard::{FunctionInfo, LanguageStandard};
use tree_sitter::Parser;
use tree_sitter_typescript;

pub struct TsBackend;

impl LanguageStandard for TsBackend {
    fn find_empty_function_at_cursor(
        &self,
        source_code: &str,
        cursor_byte: usize,
    ) -> Option<FunctionInfo> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            .expect("Failed to load Typescript grammar");

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
            .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            .expect("Error loading Typescript grammar");

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
            let doc_comment = extract_doc_comment(&node, source_code, "///");

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
