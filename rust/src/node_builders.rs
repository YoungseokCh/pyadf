use scraper::Node;

use crate::adf_node::{AdfNode, Mark, NodeKind};

pub fn doc_node(children: Vec<AdfNode>) -> AdfNode {
    AdfNode { kind: NodeKind::Doc, children }
}

pub fn text_node(text: &str, marks: Vec<Mark>) -> AdfNode {
    AdfNode {
        kind: NodeKind::Text { text: text.to_string(), marks },
        children: vec![],
    }
}

pub fn mark(mark_type: &str) -> Mark {
    Mark { mark_type: mark_type.to_string(), href: None, color: None }
}

pub fn link_mark(href: &str) -> Mark {
    Mark { mark_type: "link".to_string(), href: Some(href.to_string()), color: None }
}

pub fn color_mark(color: &str) -> Mark {
    Mark { mark_type: "textColor".to_string(), href: None, color: Some(color.to_string()) }
}

pub fn paragraph_node(children: Vec<AdfNode>) -> AdfNode {
    AdfNode { kind: NodeKind::Paragraph, children }
}

pub fn list_item_node(children: Vec<AdfNode>) -> AdfNode {
    AdfNode { kind: NodeKind::ListItem, children }
}

/// Collapse HTML whitespace: newlines/tabs become spaces, runs of spaces become one.
pub fn collapse_whitespace(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_space = false;
    for ch in s.chars() {
        if ch.is_ascii_whitespace() {
            if !prev_space {
                result.push(' ');
                prev_space = true;
            }
        } else {
            result.push(ch);
            prev_space = false;
        }
    }
    result
}

/// Extract language and text content from a <pre> element.
/// Looks for a direct <code> child with a language class.
pub fn extract_pre_content(node_ref: &ego_tree::NodeRef<Node>) -> (Option<String>, String) {
    for child in node_ref.children() {
        if let Node::Element(elem) = child.value() {
            if elem.name.local.as_ref() == "code" {
                let language = extract_language_from_class(elem);
                let text = collect_text_content(&child);
                return (language, text);
            }
        }
    }
    let text = collect_text_content(node_ref);
    (None, text)
}

/// Extract language from class="language-xxx" or class="xxx" on a <code> element.
fn extract_language_from_class(elem: &scraper::node::Element) -> Option<String> {
    let class = elem.attr("class")?;
    for cls in class.split_whitespace() {
        if let Some(lang) = cls.strip_prefix("language-") {
            if !lang.is_empty() {
                return Some(lang.to_string());
            }
        }
    }
    let first = class.split_whitespace().next()?;
    if !first.is_empty() {
        Some(first.to_string())
    } else {
        None
    }
}

/// Collect all text content from a node, preserving whitespace as-is (for <pre>).
fn collect_text_content(node_ref: &ego_tree::NodeRef<Node>) -> String {
    let mut text = String::new();
    for descendant in node_ref.descendants() {
        if let Node::Text(t) = descendant.value() {
            text.push_str(&t.text);
        }
    }
    text.trim_matches('\n').to_string()
}

/// Extract color from style="color:xxx" or style="color: xxx".
pub fn extract_color_from_style(elem: &scraper::node::Element) -> Option<String> {
    let style = elem.attr("style")?;
    for part in style.split(';') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix("color:") {
            let color = value.trim();
            if !color.is_empty() {
                return Some(color.to_string());
            }
        }
    }
    None
}

/// Check if a node is an inline ADF node (Text, HardBreak).
fn is_inline(node: &AdfNode) -> bool {
    matches!(
        node.kind,
        NodeKind::Text { .. } | NodeKind::HardBreak
    )
}

/// Wrap a sequence of inline nodes in a Paragraph. Block nodes pass through.
pub fn wrap_inline_in_paragraph(nodes: Vec<AdfNode>) -> Vec<AdfNode> {
    let mut result = Vec::new();
    let mut inline_buf = Vec::new();

    for node in nodes {
        if is_inline(&node) {
            inline_buf.push(node);
        } else {
            if !inline_buf.is_empty() {
                result.push(AdfNode {
                    kind: NodeKind::Paragraph,
                    children: std::mem::take(&mut inline_buf),
                });
            }
            result.push(node);
        }
    }
    if !inline_buf.is_empty() {
        result.push(AdfNode {
            kind: NodeKind::Paragraph,
            children: inline_buf,
        });
    }
    result
}
