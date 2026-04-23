use crate::adf_node::{AdfNode, Mark, NodeKind};
use serde_json::{Map, Value, json};

pub fn to_value(node: &AdfNode) -> Value {
    let mut obj = Map::new();

    match &node.kind {
        NodeKind::Doc => {
            obj.insert("type".to_string(), json!("doc"));
            insert_children(&mut obj, &node.children);
        }
        NodeKind::Paragraph => {
            obj.insert("type".to_string(), json!("paragraph"));
            insert_children(&mut obj, &node.children);
        }
        NodeKind::Text { text, marks } => {
            obj.insert("type".to_string(), json!("text"));
            obj.insert("text".to_string(), json!(text));
            if !marks.is_empty() {
                obj.insert(
                    "marks".to_string(),
                    Value::Array(marks.iter().map(mark_to_value).collect()),
                );
            }
        }
        NodeKind::HardBreak => {
            obj.insert("type".to_string(), json!("hardBreak"));
        }
        NodeKind::BulletList => container(&mut obj, "bulletList", &node.children),
        NodeKind::OrderedList => container(&mut obj, "orderedList", &node.children),
        NodeKind::TaskList => container(&mut obj, "taskList", &node.children),
        NodeKind::ListItem => container(&mut obj, "listItem", &node.children),
        NodeKind::TaskItem => container(&mut obj, "taskItem", &node.children),
        NodeKind::Panel => container(&mut obj, "panel", &node.children),
        NodeKind::Blockquote => container(&mut obj, "blockquote", &node.children),
        NodeKind::Table => container(&mut obj, "table", &node.children),
        NodeKind::TableRow => container(&mut obj, "tableRow", &node.children),
        NodeKind::TableHeader { colspan } => {
            container(&mut obj, "tableHeader", &node.children);
            if *colspan != 1 {
                obj.insert("attrs".to_string(), json!({ "colspan": colspan }));
            }
        }
        NodeKind::TableCell { colspan } => {
            container(&mut obj, "tableCell", &node.children);
            if *colspan != 1 {
                obj.insert("attrs".to_string(), json!({ "colspan": colspan }));
            }
        }
        NodeKind::CodeBlock { language } => {
            container(&mut obj, "codeBlock", &node.children);
            if let Some(language) = language {
                obj.insert("attrs".to_string(), json!({ "language": language }));
            }
        }
        NodeKind::InlineCard { url, data } => {
            obj.insert("type".to_string(), json!("inlineCard"));
            insert_attrs(&mut obj, card_attrs(url, data));
        }
        NodeKind::Heading { level } => {
            container(&mut obj, "heading", &node.children);
            obj.insert("attrs".to_string(), json!({ "level": level }));
        }
        NodeKind::Status { text } => {
            obj.insert("type".to_string(), json!("status"));
            obj.insert("attrs".to_string(), json!({ "text": text }));
        }
        NodeKind::Emoji { short_name, text } => {
            obj.insert("type".to_string(), json!("emoji"));
            let mut attrs = Map::new();
            attrs.insert("shortName".to_string(), json!(short_name));
            if let Some(text) = text {
                attrs.insert("text".to_string(), json!(text));
            }
            obj.insert("attrs".to_string(), Value::Object(attrs));
        }
        NodeKind::Mention { text } => {
            obj.insert("type".to_string(), json!("mention"));
            if let Some(text) = text {
                obj.insert("attrs".to_string(), json!({ "text": text }));
            }
        }
        NodeKind::BlockCard { url, data } => {
            obj.insert("type".to_string(), json!("blockCard"));
            insert_attrs(&mut obj, card_attrs(url, data));
        }
        NodeKind::KnownUnsupported {
            node_type,
            params,
            ..
        } => {
            obj.insert("type".to_string(), json!(node_type));
            if let Some(attrs) = parse_params(params) {
                obj.insert("attrs".to_string(), Value::Object(attrs));
            }
            insert_children(&mut obj, &node.children);
        }
    }

    Value::Object(obj)
}

fn container(obj: &mut Map<String, Value>, type_name: &str, children: &[AdfNode]) {
    obj.insert("type".to_string(), json!(type_name));
    insert_children(obj, children);
}

fn insert_children(obj: &mut Map<String, Value>, children: &[AdfNode]) {
    if !children.is_empty() {
        obj.insert(
            "content".to_string(),
            Value::Array(children.iter().map(to_value).collect()),
        );
    }
}

fn insert_attrs(obj: &mut Map<String, Value>, attrs: Map<String, Value>) {
    if !attrs.is_empty() {
        obj.insert("attrs".to_string(), Value::Object(attrs));
    }
}

fn mark_to_value(mark: &Mark) -> Value {
    let mut obj = Map::new();
    obj.insert("type".to_string(), json!(mark.mark_type));
    if let Some(href) = &mark.href {
        obj.insert("attrs".to_string(), json!({ "href": href }));
    }
    Value::Object(obj)
}

fn card_attrs(url: &Option<String>, data: &Option<String>) -> Map<String, Value> {
    let mut attrs = Map::new();
    if let Some(url) = url {
        attrs.insert("url".to_string(), json!(url));
    }
    if let Some(data) = data {
        attrs.insert("data".to_string(), json!(data));
    }
    attrs
}

fn parse_params(params: &Option<String>) -> Option<Map<String, Value>> {
    let raw = params.as_ref()?;
    let value: Value = serde_json::from_str(raw).ok()?;
    value.as_object().cloned()
}
