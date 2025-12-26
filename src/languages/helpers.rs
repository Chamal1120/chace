use tree_sitter::Node;

/// Extracts the text content for a given tree-sitter node
pub fn text_for<'a>(source: &'a str, node: &Node) -> &'a str {
    &source[node.start_byte()..node.end_byte()]
}

/// Extracts function signature (everything before the body)
pub fn extract_signature(node: &Node, source: &str) -> String {
    let body_node = find_body_recursive(*node);

    let start = node.start_byte();
    let end = match body_node {
        Some(b) => b.start_byte(),
        None => node.end_byte(),
    };

    source[start..end].trim().to_string()
}

/// Recursive helper to find a "body" field anywhere inside a node
fn find_body_recursive(node: Node) -> Option<Node> {
    if let Some(body) = node.child_by_field_name("body") {
        return Some(body);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(found) = find_body_recursive(child) {
            return Some(found);
        }
    }
    None
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
        let kind = sibling.kind();
        if kind == "line_comment" || kind == "comment" {
            let text = text_for(source, &sibling).trim().to_string();

            if text.starts_with(comment_prefix) {
                if text.starts_with("/**") {
                    let cleaned = clean_jsdoc(&text);
                    comments.push(cleaned);
                } else {
                    comments.push(
                        text.trim_start_matches(comment_prefix)
                            .trim()
                            .to_string(),
                    );
                }
            } else {
                break;
            }
        } else {
            break;
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

fn clean_jsdoc(text: &str) -> String {
    text.trim_start_matches("/**")
        .trim_end_matches("*/")
        .lines()
        .map(|line| {
            line.trim().trim_start_matches("*").trim().to_string()
        })
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

