use crate::adf_node::{AdfNode, Mark, NodeKind};
use serde_json::{json, Value};

/// Serialize an ADF node tree back to a serde_json::Value matching ADF JSON schema.
pub fn serialize_to_value(node: &AdfNode) -> Value {
    match &node.kind {
        NodeKind::Unknown => Value::Null,
        _ => serialize_node(node),
    }
}

fn serialize_node(node: &AdfNode) -> Value {
    let mut obj = serde_json::Map::new();

    let (type_str, attrs) = node_type_and_attrs(&node.kind);
    obj.insert("type".to_string(), Value::String(type_str.to_string()));

    // Add type-specific fields (text node has "text" at top level)
    if let NodeKind::Text { text, marks } = &node.kind {
        obj.insert("text".to_string(), Value::String(text.clone()));
        if !marks.is_empty() {
            let marks_arr: Vec<Value> = marks.iter().map(serialize_mark).collect();
            obj.insert("marks".to_string(), Value::Array(marks_arr));
        }
    }

    if let Some(attrs_val) = attrs {
        obj.insert("attrs".to_string(), attrs_val);
    }

    // Children -> "content" array (omit if empty)
    let content = serialize_children(&node.children);
    if !content.is_empty() {
        obj.insert("content".to_string(), Value::Array(content));
    }

    Value::Object(obj)
}

fn serialize_children(children: &[AdfNode]) -> Vec<Value> {
    children
        .iter()
        .filter_map(|child| {
            if matches!(child.kind, NodeKind::Unknown) {
                None
            } else {
                Some(serialize_node(child))
            }
        })
        .collect()
}

fn node_type_and_attrs(kind: &NodeKind) -> (&'static str, Option<Value>) {
    match kind {
        NodeKind::Doc => ("doc", Some(json!({"version": 1}))),
        NodeKind::Paragraph => ("paragraph", None),
        NodeKind::Text { .. } => ("text", None),
        NodeKind::HardBreak => ("hardBreak", None),
        NodeKind::BulletList => ("bulletList", None),
        NodeKind::OrderedList => ("orderedList", None),
        NodeKind::TaskList => ("taskList", None),
        NodeKind::ListItem => ("listItem", None),
        NodeKind::TaskItem => ("taskItem", None),
        NodeKind::Panel => ("panel", None),
        NodeKind::Blockquote => ("blockquote", None),
        NodeKind::Table => ("table", None),
        NodeKind::TableRow => ("tableRow", None),
        NodeKind::TableHeader { colspan } => {
            ("tableHeader", Some(json!({"colspan": colspan})))
        }
        NodeKind::TableCell { colspan } => {
            ("tableCell", Some(json!({"colspan": colspan})))
        }
        NodeKind::CodeBlock { language } => {
            let attrs = language.as_ref().map(|l| json!({"language": l}));
            ("codeBlock", attrs)
        }
        NodeKind::InlineCard { url, data } => {
            let mut attrs = serde_json::Map::new();
            if let Some(u) = url {
                attrs.insert("url".to_string(), Value::String(u.clone()));
            }
            if let Some(d) = data {
                attrs.insert("data".to_string(), Value::String(d.clone()));
            }
            let attrs_val = if attrs.is_empty() {
                None
            } else {
                Some(Value::Object(attrs))
            };
            ("inlineCard", attrs_val)
        }
        NodeKind::Heading { level } => {
            ("heading", Some(json!({"level": level})))
        }
        NodeKind::Status { text } => {
            ("status", Some(json!({"text": text})))
        }
        NodeKind::Emoji { short_name, text } => {
            let mut attrs = serde_json::Map::new();
            attrs.insert("shortName".to_string(), Value::String(short_name.clone()));
            if let Some(t) = text {
                attrs.insert("text".to_string(), Value::String(t.clone()));
            }
            ("emoji", Some(Value::Object(attrs)))
        }
        NodeKind::Mention { text } => {
            let attrs = text.as_ref().map(|t| json!({"text": t}));
            ("mention", attrs)
        }
        NodeKind::Unknown => ("unknown", None),
    }
}

fn serialize_mark(mark: &Mark) -> Value {
    let mut obj = serde_json::Map::new();
    obj.insert("type".to_string(), Value::String(mark.mark_type.clone()));

    let mut attrs = serde_json::Map::new();
    if let Some(href) = &mark.href {
        attrs.insert("href".to_string(), Value::String(href.clone()));
    }
    if let Some(color) = &mark.color {
        attrs.insert("color".to_string(), Value::String(color.clone()));
    }
    if !attrs.is_empty() {
        obj.insert("attrs".to_string(), Value::Object(attrs));
    }

    Value::Object(obj)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adf_node::parse_adf;

    #[test]
    fn round_trip_simple_paragraph() {
        let json = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Hello"}]}]}"#;
        let node = parse_adf(json).unwrap();
        let value = serialize_to_value(&node);
        let obj = value.as_object().unwrap();
        assert_eq!(obj["type"], "doc");
        assert_eq!(obj["attrs"]["version"], 1);
        let content = obj["content"].as_array().unwrap();
        assert_eq!(content[0]["type"], "paragraph");
        assert_eq!(content[0]["content"][0]["text"], "Hello");
    }

    #[test]
    fn round_trip_bold_text() {
        let json = r#"{"type":"text","text":"bold","marks":[{"type":"strong"}]}"#;
        let node = parse_adf(json).unwrap();
        let value = serialize_to_value(&node);
        let obj = value.as_object().unwrap();
        assert_eq!(obj["marks"][0]["type"], "strong");
    }

    #[test]
    fn round_trip_link() {
        let json = r#"{"type":"text","text":"click","marks":[{"type":"link","attrs":{"href":"http://example.com"}}]}"#;
        let node = parse_adf(json).unwrap();
        let value = serialize_to_value(&node);
        let obj = value.as_object().unwrap();
        assert_eq!(obj["marks"][0]["attrs"]["href"], "http://example.com");
    }

    #[test]
    fn skip_unknown_nodes() {
        let json = r#"{"type":"doc","content":[{"type":"mediaSingle"}]}"#;
        let node = parse_adf(json).unwrap();
        let value = serialize_to_value(&node);
        let content = value.as_object().unwrap().get("content");
        // Unknown nodes are filtered out
        assert!(content.is_none());
    }

    #[test]
    fn heading_with_level() {
        let json = r#"{"type":"heading","attrs":{"level":3},"content":[{"type":"text","text":"H3"}]}"#;
        let node = parse_adf(json).unwrap();
        let value = serialize_to_value(&node);
        assert_eq!(value["attrs"]["level"], 3);
    }

    #[test]
    fn code_block_with_language() {
        let json = r#"{"type":"codeBlock","attrs":{"language":"rust"},"content":[{"type":"text","text":"fn main(){}"}]}"#;
        let node = parse_adf(json).unwrap();
        let value = serialize_to_value(&node);
        assert_eq!(value["attrs"]["language"], "rust");
    }
}
