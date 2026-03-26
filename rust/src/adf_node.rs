use crate::errors::AdfError;
use serde_json::Value;

/// A text formatting mark.
#[derive(Debug, Clone)]
pub struct Mark {
    pub mark_type: String,
    pub href: Option<String>,
    pub color: Option<String>,
}

/// Node-specific data for each ADF node type.
#[derive(Debug, Clone)]
pub enum NodeKind {
    Doc,
    Paragraph,
    Text {
        text: String,
        marks: Vec<Mark>,
    },
    HardBreak,
    BulletList,
    OrderedList,
    TaskList,
    ListItem,
    TaskItem,
    Panel,
    Blockquote,
    Table,
    TableRow,
    TableHeader {
        colspan: u32,
    },
    TableCell {
        colspan: u32,
    },
    CodeBlock {
        language: Option<String>,
    },
    InlineCard {
        url: Option<String>,
        data: Option<String>,
    },
    Heading {
        level: u8,
    },
    Status {
        text: String,
    },
    Emoji {
        short_name: String,
        text: Option<String>,
    },
    Mention {
        text: Option<String>,
    },
    Unknown,
}

/// An ADF node: tree structure (children) + type-specific data (kind).
#[derive(Debug, Clone)]
pub struct AdfNode {
    pub kind: NodeKind,
    pub children: Vec<AdfNode>,
}

/// Supported ADF type strings (static, no allocation).
const SUPPORTED_TYPES: &[&str] = &[
    "doc",
    "paragraph",
    "text",
    "hardBreak",
    "bulletList",
    "listItem",
    "panel",
    "table",
    "tableRow",
    "tableHeader",
    "tableCell",
    "codeBlock",
    "inlineCard",
    "taskList",
    "taskItem",
    "orderedList",
    "heading",
    "blockquote",
    "status",
    "emoji",
    "mention",
];

/// Convert to Vec<String> only when needed (error messages).
fn supported_type_strings() -> Vec<String> {
    SUPPORTED_TYPES.iter().map(|s| String::from(*s)).collect()
}

/// Known unsupported types that are silently skipped (not errors).
const KNOWN_UNSUPPORTED: &[&str] = &[
    "mediaSingle",
    "mediaGroup",
    "mediaInline",
    "expand",
    "rule",
    "media",
    "embedCard",
];

/// Parse a JSON string into an ADF node tree.
pub fn parse_adf(json: &str) -> Result<AdfNode, AdfError> {
    let value: Value = serde_json::from_str(json).map_err(|e| {
        // Compute absolute byte offset from line/column.
        // serde_json reports 1-based line and column numbers.
        let position = compute_byte_offset(json, e.line(), e.column());
        AdfError::InvalidJson {
            message: e.to_string(),
            position,
        }
    })?;

    parse_adf_value(&value, "")
}

/// Parse a pre-built serde_json::Value into an ADF node tree.
pub fn parse_adf_value(value: &Value, node_path: &str) -> Result<AdfNode, AdfError> {
    match value {
        Value::Object(_) => parse_node(value, node_path),
        _ => Err(AdfError::InvalidInput {
            expected_type: "JSON object, dict, or None".to_string(),
            actual_type: json_type_name(value).to_string(),
        }),
    }
}

/// Compute the absolute 0-based byte offset from 1-based line and column numbers.
fn compute_byte_offset(input: &str, line: usize, column: usize) -> Option<usize> {
    let mut offset = 0;
    for (i, text_line) in input.split('\n').enumerate() {
        if i + 1 == line {
            // column is 1-based
            let col_offset = column.saturating_sub(1);
            return Some(offset + col_offset);
        }
        offset += text_line.len() + 1; // +1 for the '\n'
    }
    None
}

fn json_type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "str",
        Value::Array(_) => "list",
        Value::Object(_) => "dict",
    }
}

fn parse_node(value: &Value, node_path: &str) -> Result<AdfNode, AdfError> {
    let obj = value.as_object().ok_or_else(|| AdfError::InvalidField {
        field_name: "node".to_string(),
        invalid_value: json_type_name(value).to_string(),
        node_type: None,
        node_path: Some(node_path_or_root(node_path)),
        expected_values: None,
    })?;

    let type_val = obj.get("type").ok_or_else(|| AdfError::MissingField {
        field_name: "type".to_string(),
        node_type: None,
        node_path: Some(node_path_or_root(node_path)),
        expected_values: Some(supported_type_strings()),
    })?;

    let type_str = type_val.as_str().ok_or_else(|| AdfError::InvalidField {
        field_name: "type".to_string(),
        invalid_value: format!("{type_val:?}"),
        node_type: None,
        node_path: Some(node_path_or_root(node_path)),
        expected_values: Some(supported_type_strings()),
    })?;

    let current_path = if node_path.is_empty() {
        type_str.to_string()
    } else {
        format!("{node_path} > {type_str}")
    };

    // Validate attrs shape (must be object or absent)
    validate_attrs(obj, &current_path)?;

    // Parse children
    let children = parse_children(obj, type_str, &current_path)?;

    // Build NodeKind from type string + obj fields (reads attrs by reference, no clone)
    let attrs = obj
        .get("attrs")
        .and_then(|v| v.as_object())
        .unwrap_or(&EMPTY_MAP);
    let kind = build_node_kind(type_str, attrs, obj, &current_path)?;

    Ok(AdfNode { kind, children })
}

/// Empty map constant to avoid allocation when attrs is absent.
static EMPTY_MAP: std::sync::LazyLock<serde_json::Map<String, Value>> =
    std::sync::LazyLock::new(serde_json::Map::new);

fn validate_attrs(obj: &serde_json::Map<String, Value>, node_path: &str) -> Result<(), AdfError> {
    match obj.get("attrs") {
        Some(Value::Object(_)) | Some(Value::Null) | None => Ok(()),
        Some(other) => Err(AdfError::InvalidField {
            field_name: "attrs".to_string(),
            invalid_value: format!("{other:?}"),
            node_type: None,
            node_path: Some(node_path_or_root(node_path)),
            expected_values: None,
        }),
    }
}

fn parse_children(
    obj: &serde_json::Map<String, Value>,
    type_str: &str,
    node_path: &str,
) -> Result<Vec<AdfNode>, AdfError> {
    match obj.get("content") {
        Some(Value::Array(arr)) => {
            let mut kids = Vec::with_capacity(arr.len());
            for (idx, child_val) in arr.iter().enumerate() {
                if !child_val.is_object() {
                    return Err(AdfError::InvalidField {
                        field_name: "content".to_string(),
                        invalid_value: format!("{child_val:?}"),
                        node_type: None,
                        node_path: Some(node_path_or_root(node_path)),
                        expected_values: None,
                    });
                }
                let child_path = if node_path.is_empty() {
                    type_str.to_string()
                } else {
                    format!("{node_path} > {type_str}[{idx}]")
                };
                kids.push(parse_node(child_val, &child_path)?);
            }
            Ok(kids)
        }
        Some(Value::Null) | None => Ok(Vec::new()),
        Some(other) => Err(AdfError::InvalidField {
            field_name: "content".to_string(),
            invalid_value: format!("{other:?}"),
            node_type: None,
            node_path: Some(node_path_or_root(node_path)),
            expected_values: None,
        }),
    }
}

fn build_node_kind(
    type_str: &str,
    attrs: &serde_json::Map<String, Value>,
    obj: &serde_json::Map<String, Value>,
    current_path: &str,
) -> Result<NodeKind, AdfError> {
    if KNOWN_UNSUPPORTED.contains(&type_str) {
        return Ok(NodeKind::Unknown);
    }

    match type_str {
        "doc" => Ok(NodeKind::Doc),
        "paragraph" => Ok(NodeKind::Paragraph),
        "text" => {
            let text = obj
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let marks = parse_marks(obj, current_path)?;
            Ok(NodeKind::Text { text, marks })
        }
        "hardBreak" => Ok(NodeKind::HardBreak),
        "bulletList" => Ok(NodeKind::BulletList),
        "orderedList" => Ok(NodeKind::OrderedList),
        "taskList" => Ok(NodeKind::TaskList),
        "listItem" => Ok(NodeKind::ListItem),
        "taskItem" => Ok(NodeKind::TaskItem),
        "panel" => Ok(NodeKind::Panel),
        "blockquote" => Ok(NodeKind::Blockquote),
        "table" => Ok(NodeKind::Table),
        "tableRow" => Ok(NodeKind::TableRow),
        "tableHeader" => {
            let colspan = attr_u32(attrs, "colspan", 1);
            Ok(NodeKind::TableHeader { colspan })
        }
        "tableCell" => {
            let colspan = attr_u32(attrs, "colspan", 1);
            Ok(NodeKind::TableCell { colspan })
        }
        "codeBlock" => {
            let language = attr_opt_string(attrs, "language");
            Ok(NodeKind::CodeBlock { language })
        }
        "inlineCard" => {
            let url = attr_opt_string(attrs, "url");
            let data = attr_opt_string(attrs, "data");
            Ok(NodeKind::InlineCard { url, data })
        }
        "heading" => {
            let level = attrs
                .get("level")
                .and_then(|v| v.as_i64())
                .unwrap_or(1)
                .clamp(1, 6) as u8;
            Ok(NodeKind::Heading { level })
        }
        "status" => {
            let text = attr_string(attrs, "text");
            Ok(NodeKind::Status { text })
        }
        "emoji" => {
            let short_name = attr_string(attrs, "shortName");
            let text = attr_opt_string(attrs, "text");
            Ok(NodeKind::Emoji { short_name, text })
        }
        "mention" => {
            let text = attr_opt_string(attrs, "text");
            Ok(NodeKind::Mention { text })
        }
        _ => Err(AdfError::UnsupportedNodeType {
            node_type: type_str.to_string(),
            node_path: Some(current_path.to_string()),
            supported_types: Some(supported_type_strings()),
        }),
    }
}

fn attr_string(attrs: &serde_json::Map<String, Value>, key: &str) -> String {
    attrs
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn attr_opt_string(attrs: &serde_json::Map<String, Value>, key: &str) -> Option<String> {
    attrs.get(key).and_then(|v| v.as_str()).map(String::from)
}

fn attr_u32(attrs: &serde_json::Map<String, Value>, key: &str, default: u32) -> u32 {
    attrs
        .get(key)
        .and_then(|v| v.as_i64())
        .map(|v| v.clamp(1, 1000) as u32)
        .unwrap_or(default)
}

fn parse_marks(
    obj: &serde_json::Map<String, Value>,
    node_path: &str,
) -> Result<Vec<Mark>, AdfError> {
    let marks_val = match obj.get("marks") {
        Some(v) => v,
        None => return Ok(Vec::new()),
    };

    let arr = marks_val.as_array().ok_or_else(|| AdfError::InvalidField {
        field_name: "marks".to_string(),
        invalid_value: format!("{marks_val:?}"),
        node_type: None,
        node_path: Some(node_path_or_root(node_path)),
        expected_values: None,
    })?;

    let mut marks = Vec::with_capacity(arr.len());
    for mark_val in arr {
        let mark_obj = mark_val.as_object().ok_or_else(|| AdfError::InvalidField {
            field_name: "marks".to_string(),
            invalid_value: format!("{marks_val:?}"),
            node_type: None,
            node_path: Some(node_path_or_root(node_path)),
            expected_values: None,
        })?;

        let mark_type = mark_obj
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let attrs = mark_obj.get("attrs").and_then(|a| a.as_object());

        let href = attrs
            .and_then(|a| a.get("href"))
            .and_then(|v| v.as_str())
            .map(String::from);

        let color = attrs
            .and_then(|a| a.get("color"))
            .and_then(|v| v.as_str())
            .map(String::from);

        marks.push(Mark { mark_type, href, color });
    }

    Ok(marks)
}

fn node_path_or_root(path: &str) -> String {
    if path.is_empty() {
        "<root>".to_string()
    } else {
        path.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_paragraph() {
        let json = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Hello"}]}]}"#;
        let node = parse_adf(json).unwrap();
        assert!(matches!(node.kind, NodeKind::Doc));
        assert_eq!(node.children.len(), 1);
        assert!(matches!(node.children[0].kind, NodeKind::Paragraph));
        match &node.children[0].children[0].kind {
            NodeKind::Text { text, .. } => assert_eq!(text, "Hello"),
            other => panic!("Expected Text, got: {other:?}"),
        }
    }

    #[test]
    fn parse_invalid_json() {
        let result = parse_adf("not json");
        assert!(result.is_err());
        match result.unwrap_err() {
            AdfError::InvalidJson { .. } => {}
            other => panic!("Expected InvalidJson, got: {other:?}"),
        }
    }

    #[test]
    fn parse_missing_type() {
        let result = parse_adf(r#"{"content":[]}"#);
        assert!(result.is_err());
        match result.unwrap_err() {
            AdfError::MissingField { field_name, .. } => assert_eq!(field_name, "type"),
            other => panic!("Expected MissingField, got: {other:?}"),
        }
    }

    #[test]
    fn parse_unsupported_type() {
        let result = parse_adf(r#"{"type":"totallyFake"}"#);
        assert!(result.is_err());
        match result.unwrap_err() {
            AdfError::UnsupportedNodeType { node_type, .. } => {
                assert_eq!(node_type, "totallyFake")
            }
            other => panic!("Expected UnsupportedNodeType, got: {other:?}"),
        }
    }

    #[test]
    fn parse_known_unsupported_silently() {
        let json = r#"{"type":"doc","content":[{"type":"mediaSingle"}]}"#;
        let node = parse_adf(json).unwrap();
        assert_eq!(node.children.len(), 1);
        assert!(matches!(node.children[0].kind, NodeKind::Unknown));
    }

    #[test]
    fn parse_bold_text() {
        let json = r#"{"type":"text","text":"bold","marks":[{"type":"strong"}]}"#;
        let node = parse_adf(json).unwrap();
        match &node.kind {
            NodeKind::Text { marks, .. } => {
                assert_eq!(marks.len(), 1);
                assert_eq!(marks[0].mark_type, "strong");
            }
            other => panic!("Expected Text, got: {other:?}"),
        }
    }

    #[test]
    fn parse_link_text() {
        let json = r#"{"type":"text","text":"click","marks":[{"type":"link","attrs":{"href":"http://example.com"}}]}"#;
        let node = parse_adf(json).unwrap();
        match &node.kind {
            NodeKind::Text { marks, .. } => {
                assert_eq!(marks[0].mark_type, "link");
                assert_eq!(marks[0].href.as_deref(), Some("http://example.com"));
            }
            other => panic!("Expected Text, got: {other:?}"),
        }
    }

    #[test]
    fn parse_heading() {
        let json =
            r#"{"type":"heading","attrs":{"level":2},"content":[{"type":"text","text":"Title"}]}"#;
        let node = parse_adf(json).unwrap();
        match &node.kind {
            NodeKind::Heading { level } => assert_eq!(*level, 2),
            other => panic!("Expected Heading, got: {other:?}"),
        }
    }

    #[test]
    fn parse_invalid_attrs() {
        let result = parse_adf(r#"{"type":"doc","attrs":"bad"}"#);
        assert!(result.is_err());
        match result.unwrap_err() {
            AdfError::InvalidField { field_name, .. } => assert_eq!(field_name, "attrs"),
            other => panic!("Expected InvalidField, got: {other:?}"),
        }
    }

    #[test]
    fn parse_invalid_content() {
        let result = parse_adf(r#"{"type":"doc","content":"bad"}"#);
        assert!(result.is_err());
        match result.unwrap_err() {
            AdfError::InvalidField { field_name, .. } => assert_eq!(field_name, "content"),
            other => panic!("Expected InvalidField, got: {other:?}"),
        }
    }

    #[test]
    fn parse_invalid_marks() {
        let result = parse_adf(r#"{"type":"text","text":"x","marks":"bad"}"#);
        assert!(result.is_err());
        match result.unwrap_err() {
            AdfError::InvalidField { field_name, .. } => assert_eq!(field_name, "marks"),
            other => panic!("Expected InvalidField, got: {other:?}"),
        }
    }
}
