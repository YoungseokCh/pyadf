use crate::adf_node::{AdfNode, Mark, NodeKind};

/// Render an ADF node tree to Jira wiki markup.
pub fn render_jira(node: &AdfNode) -> String {
    let mut out = String::new();
    let ctx = JiraContext { list_depth: 0, list_type: ListType::None };
    render_node(node, &ctx, &mut out);
    out
}

#[derive(Clone, Copy)]
enum ListType {
    None,
    Bullet,
    Ordered,
    Task,
}

struct JiraContext {
    list_depth: usize,
    list_type: ListType,
}

fn render_node(node: &AdfNode, ctx: &JiraContext, out: &mut String) {
    match &node.kind {
        NodeKind::Doc => render_doc(node, ctx, out),
        NodeKind::Paragraph => render_children(node, ctx, out),
        NodeKind::Text { text, marks } => render_text(text, marks, out),
        NodeKind::HardBreak => out.push_str("\\\\"),
        NodeKind::Heading { level } => render_heading(node, *level, ctx, out),
        NodeKind::BulletList => render_list(node, ListType::Bullet, ctx, out),
        NodeKind::OrderedList => render_list(node, ListType::Ordered, ctx, out),
        NodeKind::TaskList => render_list(node, ListType::Task, ctx, out),
        NodeKind::ListItem | NodeKind::TaskItem => render_list_item(node, ctx, out),
        NodeKind::CodeBlock { language } => render_code_block(node, language.as_deref(), out),
        NodeKind::Blockquote => render_blockquote(node, ctx, out),
        NodeKind::Panel => render_blockquote(node, ctx, out),
        NodeKind::Table => render_table(node, ctx, out),
        NodeKind::TableRow => {} // handled by render_table
        NodeKind::TableHeader { .. } | NodeKind::TableCell { .. } => {
            render_children(node, ctx, out);
        }
        NodeKind::Status { text } => {
            out.push_str("{status:color=green}");
            out.push_str(text);
            out.push_str("{status}");
        }
        NodeKind::Emoji { short_name, text } => {
            out.push_str(text.as_deref().unwrap_or(short_name));
        }
        NodeKind::Mention { text } => {
            out.push_str(text.as_deref().unwrap_or(""));
        }
        NodeKind::InlineCard { url, .. } => {
            if let Some(u) = url {
                out.push('[');
                out.push_str(u);
                out.push(']');
            }
        }
        NodeKind::Unknown => render_children(node, ctx, out),
    }
}

fn render_doc(node: &AdfNode, ctx: &JiraContext, out: &mut String) {
    let mut parts: Vec<String> = Vec::new();
    for child in &node.children {
        let mut child_out = String::new();
        render_node(child, ctx, &mut child_out);
        if !child_out.is_empty() {
            parts.push(child_out);
        }
    }
    out.push_str(&parts.join("\n\n"));
}

fn render_heading(node: &AdfNode, level: u8, ctx: &JiraContext, out: &mut String) {
    out.push_str(&format!("h{level}. "));
    render_children(node, ctx, out);
}

fn render_list(node: &AdfNode, lt: ListType, ctx: &JiraContext, out: &mut String) {
    let child_ctx = JiraContext {
        list_depth: ctx.list_depth + 1,
        list_type: lt,
    };
    let mut items: Vec<String> = Vec::new();
    for child in &node.children {
        let mut item_out = String::new();
        render_list_item_line(child, &child_ctx, &mut item_out);
        items.push(item_out);
    }
    out.push_str(&items.join("\n"));
}

fn render_list_item_line(node: &AdfNode, ctx: &JiraContext, out: &mut String) {
    let marker = match ctx.list_type {
        ListType::Bullet | ListType::Task => "*",
        ListType::Ordered => "#",
        ListType::None => "*",
    };
    let prefix: String = marker.repeat(ctx.list_depth);
    out.push_str(&prefix);
    out.push(' ');
    render_list_item(node, ctx, out);
}

fn render_list_item(node: &AdfNode, ctx: &JiraContext, out: &mut String) {
    for child in &node.children {
        // Nested lists should be rendered inline
        match &child.kind {
            NodeKind::BulletList | NodeKind::OrderedList | NodeKind::TaskList => {
                out.push('\n');
                render_node(child, ctx, out);
            }
            _ => render_node(child, ctx, out),
        }
    }
}

fn render_text(text: &str, marks: &[Mark], out: &mut String) {
    if marks.is_empty() {
        out.push_str(text);
        return;
    }
    apply_marks(text, marks, out);
}

fn apply_marks(text: &str, marks: &[Mark], out: &mut String) {
    // Build nested formatting from outermost to innermost
    let mut result = text.to_string();
    for mark in marks {
        result = wrap_mark(&result, mark);
    }
    out.push_str(&result);
}

fn wrap_mark(text: &str, mark: &Mark) -> String {
    match mark.mark_type.as_str() {
        "strong" => format!("*{text}*"),
        "em" => format!("_{text}_"),
        "code" => format!("{{{{{text}}}}}"),
        "strike" => format!("-{text}-"),
        "underline" => format!("+{text}+"),
        "superscript" => format!("^{text}^"),
        "subsup" => format!("~{text}~"),
        "link" => {
            let href = mark.href.as_deref().unwrap_or("");
            format!("[{text}|{href}]")
        }
        "textColor" => {
            let color = mark.color.as_deref().unwrap_or("black");
            format!("{{color:{color}}}{text}{{color}}")
        }
        _ => text.to_string(),
    }
}

fn render_code_block(node: &AdfNode, language: Option<&str>, out: &mut String) {
    match language {
        Some(lang) if !lang.is_empty() => out.push_str(&format!("{{code:{lang}}}")),
        _ => out.push_str("{code}"),
    }
    out.push('\n');
    let dummy_ctx = JiraContext { list_depth: 0, list_type: ListType::None };
    render_children(node, &dummy_ctx, out);
    out.push('\n');
    out.push_str("{code}");
}

fn render_blockquote(node: &AdfNode, ctx: &JiraContext, out: &mut String) {
    out.push_str("{quote}\n");
    render_children(node, ctx, out);
    out.push_str("\n{quote}");
}

fn render_table(node: &AdfNode, ctx: &JiraContext, out: &mut String) {
    let mut rows: Vec<String> = Vec::new();
    for row_node in &node.children {
        if !matches!(row_node.kind, NodeKind::TableRow) {
            continue;
        }
        let mut row_out = String::new();
        render_table_row(&row_node.children, ctx, &mut row_out);
        rows.push(row_out);
    }
    out.push_str(&rows.join("\n"));
}

fn render_table_row(cells: &[AdfNode], ctx: &JiraContext, out: &mut String) {
    let is_header = cells
        .iter()
        .any(|c| matches!(c.kind, NodeKind::TableHeader { .. }));
    let sep = if is_header { "||" } else { "|" };

    out.push_str(sep);
    for cell in cells {
        let mut cell_out = String::new();
        render_children(cell, ctx, &mut cell_out);
        out.push_str(&cell_out);
        out.push_str(sep);
    }
}

fn render_children(node: &AdfNode, ctx: &JiraContext, out: &mut String) {
    for child in &node.children {
        render_node(child, ctx, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adf_node::parse_adf;

    fn jira(json: &str) -> String {
        let node = parse_adf(json).unwrap();
        render_jira(&node)
    }

    #[test]
    fn simple_paragraph() {
        let result = jira(r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Hello"}]}]}"#);
        assert_eq!(result, "Hello");
    }

    #[test]
    fn bold_text() {
        let result = jira(r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"bold","marks":[{"type":"strong"}]}]}]}"#);
        assert_eq!(result, "*bold*");
    }

    #[test]
    fn italic_text() {
        let result = jira(r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"italic","marks":[{"type":"em"}]}]}]}"#);
        assert_eq!(result, "_italic_");
    }

    #[test]
    fn heading() {
        let result = jira(r#"{"type":"heading","attrs":{"level":2},"content":[{"type":"text","text":"Title"}]}"#);
        assert_eq!(result, "h2. Title");
    }

    #[test]
    fn bullet_list() {
        let result = jira(r#"{"type":"bulletList","content":[{"type":"listItem","content":[{"type":"paragraph","content":[{"type":"text","text":"A"}]}]},{"type":"listItem","content":[{"type":"paragraph","content":[{"type":"text","text":"B"}]}]}]}"#);
        assert_eq!(result, "* A\n* B");
    }

    #[test]
    fn ordered_list() {
        let result = jira(r#"{"type":"orderedList","content":[{"type":"listItem","content":[{"type":"paragraph","content":[{"type":"text","text":"A"}]}]},{"type":"listItem","content":[{"type":"paragraph","content":[{"type":"text","text":"B"}]}]}]}"#);
        assert_eq!(result, "# A\n# B");
    }

    #[test]
    fn code_block() {
        let result = jira(r#"{"type":"codeBlock","attrs":{"language":"python"},"content":[{"type":"text","text":"print('hi')"}]}"#);
        assert_eq!(result, "{code:python}\nprint('hi')\n{code}");
    }

    #[test]
    fn link_text() {
        let result = jira(r#"{"type":"text","text":"click","marks":[{"type":"link","attrs":{"href":"http://example.com"}}]}"#);
        assert_eq!(result, "[click|http://example.com]");
    }

    #[test]
    fn blockquote() {
        let result = jira(r#"{"type":"blockquote","content":[{"type":"paragraph","content":[{"type":"text","text":"Quote"}]}]}"#);
        assert_eq!(result, "{quote}\nQuote\n{quote}");
    }

    #[test]
    fn table() {
        let json = r#"{"type":"table","content":[{"type":"tableRow","content":[{"type":"tableHeader","content":[{"type":"paragraph","content":[{"type":"text","text":"H1"}]}]},{"type":"tableHeader","content":[{"type":"paragraph","content":[{"type":"text","text":"H2"}]}]}]},{"type":"tableRow","content":[{"type":"tableCell","content":[{"type":"paragraph","content":[{"type":"text","text":"A"}]}]},{"type":"tableCell","content":[{"type":"paragraph","content":[{"type":"text","text":"B"}]}]}]}]}"#;
        let result = jira(json);
        assert_eq!(result, "||H1||H2||\n|A|B|");
    }
}
