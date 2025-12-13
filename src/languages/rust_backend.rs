use tree_sitter::{Node, Parser};
use tree_sitter_rust;

#[derive(Debug)]
pub struct FunctionInfo {
    pub signature: String,
    pub doc_comment: Option<String>,
    pub start_byte: usize,
    pub end_byte: usize,
}

pub fn find_empty_function_at_cursor(
    source_code: &str,
    cursor_byte: usize,
) -> Option<FunctionInfo> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .expect("Failed to load Rust grammar");

    let tree = parser.parse(source_code, None)?;
    let root = tree.root_node();
    let mut cursor = root.walk();

    for node in root.children(&mut cursor) {
        if node.kind() != "function_item" {
            continue;
        }

        // Check if the cursor is inside the function
        if cursor_byte < node.start_byte() || cursor_byte > node.end_byte() {
            continue;
        }

        // Check if empty
        if !is_empty_function(&node, &source_code) {
            continue;
        }

        let body_node = node.child_by_field_name("body").unwrap();

        let signature = extract_signature(&node, source_code);
        let doc_comment = extract_doc_comment(&node, source_code);

        return Some(FunctionInfo {
            signature,
            doc_comment,
            start_byte: body_node.start_byte() + 1,
            end_byte: body_node.end_byte() - 1,
        });
    }

    None
}

pub fn _find_empty_functions(source_code: &str) -> Vec<FunctionInfo> {
    let mut parser = Parser::new();

    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .expect("Error loading Rust grammar");

    let tree = parser.parse(source_code, None).unwrap();
    let root_node = tree.root_node();
    let mut cursor = root_node.walk();
    let mut funcs = Vec::new();

    for node in root_node.children(&mut cursor) {
        if node.kind() != "function_item" {
            continue;
        }

        if !is_empty_function(&node, source_code) {
            continue;
        }

        let signature = extract_signature(&node, source_code);
        let doc_comment = extract_doc_comment(&node, source_code);

        funcs.push(FunctionInfo {
            signature,
            doc_comment,
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
        });
    }

    funcs
}

fn is_empty_function(node: &Node, source: &str) -> bool {
    let body = match node.child_by_field_name("body") {
        Some(body_node) => text_for(source, &body_node),
        None => return false,
    };

    // Remove the openning and closing braces and whitespaces inside
    body.chars().all(|c| c.is_whitespace() || c == '{' || c == '}')
}

// ------------------------ Helpers/Utils ----------------------------------

fn extract_signature(node: &Node, source: &str) -> String {
    let body_node =
        node.child_by_field_name("body").expect("Function has no body");

    let start = node.start_byte();
    let end = body_node.start_byte();

    source[start..end].trim().to_string()
}

fn extract_doc_comment(node: &Node, source: &str) -> Option<String> {
    let mut comments = Vec::new();
    let mut current = node.prev_sibling();

    while let Some(sibling) = current {
        match sibling.kind() {
            "line_comment" => {
                let text = text_for(source, &sibling).trim().to_string();

                if text.starts_with("///") {
                    comments.push(
                        text.trim_start_matches("///").trim().to_string(),
                    );
                } else {
                    break;
                }
            }

            _ => {
                break;
            }
        }
        current = sibling.prev_sibling();
    }

    if comments.is_empty() {
        None
    } else {
        comments.reverse();
        Some(comments.join("\n"))
    }
}

fn text_for<'a>(source: &'a str, node: &Node) -> &'a str {
    &source[node.start_byte()..node.end_byte()]
}
