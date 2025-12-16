use tree_sitter::Node;

/// Extracts the text content for a given tree-sitter node
pub fn text_for<'a>(source: &'a str, node: &Node) -> &'a str {
    &source[node.start_byte()..node.end_byte()]
}

/// Extracts function signature (everything before the body)
pub fn extract_signature(node: &Node, source: &str) -> String {
    let body_node =
        node.child_by_field_name("body").expect("Function has no body");

    let start = node.start_byte();
    let end = body_node.start_byte();

    source[start..end].trim().to_string()
}

/// Extracts documentation comment for a node
/// Looks for comment lines starting with the given prefix (e.g., "///", "/**", "#")
pub fn extract_doc_comment(
    node: &Node,
    source: &str,
    comment_prefix: &str,
) -> Option<String> {
    let mut comments = Vec::new();
    let mut current = node.prev_sibling();

    while let Some(sibling) = current {
        match sibling.kind() {
            "line_comment" => {
                let text = text_for(source, &sibling).trim().to_string();

                if text.starts_with(comment_prefix) {
                    comments.push(
                        text.trim_start_matches(comment_prefix)
                            .trim()
                            .to_string(),
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

/// Checks if a function body contains only whitespace and braces
pub fn is_empty_body(body_text: &str) -> bool {
    body_text.chars().all(|c| c.is_whitespace() || c == '{' || c == '}')
}
