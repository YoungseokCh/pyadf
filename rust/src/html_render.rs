use crate::adf_node::{AdfNode, Mark, NodeKind};

/// Render an ADF node tree to HTML.
pub fn render_html(node: &AdfNode) -> String {
    let mut out = String::new();
    render_node(node, &mut out);
    out
}

fn render_node(node: &AdfNode, out: &mut String) {
    match &node.kind {
        NodeKind::Doc => render_children(node, out),
        NodeKind::Paragraph => wrap_tag("p", node, out),
        NodeKind::Text { text, marks } => render_text(text, marks, out),
        NodeKind::HardBreak => out.push_str("<br>"),
        NodeKind::Heading { level } => {
            let tag = format!("h{level}");
            wrap_tag(&tag, node, out);
        }
        NodeKind::BulletList => wrap_tag("ul", node, out),
        NodeKind::OrderedList => wrap_tag("ol", node, out),
        NodeKind::ListItem => wrap_tag("li", node, out),
        NodeKind::TaskList => {
            out.push_str("<ul class=\"task-list\">");
            render_children(node, out);
            out.push_str("</ul>");
        }
        NodeKind::TaskItem => {
            out.push_str("<li class=\"task-item\"><input type=\"checkbox\" disabled> ");
            render_children(node, out);
            out.push_str("</li>");
        }
        NodeKind::CodeBlock { language } => render_code_block(node, language.as_deref(), out),
        NodeKind::Blockquote => wrap_tag("blockquote", node, out),
        NodeKind::Panel => {
            out.push_str("<div class=\"panel\">");
            render_children(node, out);
            out.push_str("</div>");
        }
        NodeKind::Table => wrap_tag("table", node, out),
        NodeKind::TableRow => wrap_tag("tr", node, out),
        NodeKind::TableHeader { colspan } => render_table_cell_tag("th", *colspan, node, out),
        NodeKind::TableCell { colspan } => render_table_cell_tag("td", *colspan, node, out),
        NodeKind::Status { text } => {
            out.push_str("<span class=\"status\">");
            escape_html(text, out);
            out.push_str("</span>");
        }
        NodeKind::Emoji { short_name, text } => {
            let display = text.as_deref().unwrap_or(short_name);
            escape_html(display, out);
        }
        NodeKind::Mention { text } => {
            out.push_str("<span class=\"mention\">");
            escape_html(text.as_deref().unwrap_or(""), out);
            out.push_str("</span>");
        }
        NodeKind::InlineCard { url, .. } => {
            if let Some(u) = url {
                out.push_str("<a href=\"");
                escape_html(u, out);
                out.push_str("\">");
                escape_html(u, out);
                out.push_str("</a>");
            }
        }
        NodeKind::Unknown => render_children(node, out),
    }
}

fn render_text(text: &str, marks: &[Mark], out: &mut String) {
    let (open_tags, close_tags) = build_mark_tags(marks);
    for tag in &open_tags {
        out.push_str(tag);
    }
    escape_html(text, out);
    for tag in close_tags.iter().rev() {
        out.push_str(tag);
    }
}

fn build_mark_tags(marks: &[Mark]) -> (Vec<String>, Vec<String>) {
    let mut open = Vec::with_capacity(marks.len());
    let mut close = Vec::with_capacity(marks.len());
    for mark in marks {
        let (o, c) = mark_to_tags(mark);
        open.push(o);
        close.push(c);
    }
    (open, close)
}

fn mark_to_tags(mark: &Mark) -> (String, String) {
    match mark.mark_type.as_str() {
        "strong" => ("<strong>".to_string(), "</strong>".to_string()),
        "em" => ("<em>".to_string(), "</em>".to_string()),
        "code" => ("<code>".to_string(), "</code>".to_string()),
        "strike" => ("<del>".to_string(), "</del>".to_string()),
        "underline" => ("<u>".to_string(), "</u>".to_string()),
        "superscript" => ("<sup>".to_string(), "</sup>".to_string()),
        "subsup" => ("<sub>".to_string(), "</sub>".to_string()),
        "link" => {
            let href = mark.href.as_deref().unwrap_or("");
            let mut open = String::from("<a href=\"");
            escape_html(href, &mut open);
            open.push_str("\">");
            (open, "</a>".to_string())
        }
        "textColor" => {
            let color = mark.color.as_deref().unwrap_or("");
            let mut open = String::from("<span style=\"color:");
            escape_html(color, &mut open);
            open.push_str("\">");
            (open, "</span>".to_string())
        }
        _ => (String::new(), String::new()),
    }
}

fn render_code_block(node: &AdfNode, language: Option<&str>, out: &mut String) {
    out.push_str("<pre><code");
    if let Some(lang) = language {
        out.push_str(" class=\"language-");
        escape_html(lang, out);
        out.push('"');
    }
    out.push('>');
    // Code block children are text nodes; escape their content
    for child in &node.children {
        if let NodeKind::Text { text, .. } = &child.kind {
            escape_html(text, out);
        }
    }
    out.push_str("</code></pre>");
}

fn render_table_cell_tag(tag: &str, colspan: u32, node: &AdfNode, out: &mut String) {
    out.push('<');
    out.push_str(tag);
    if colspan > 1 {
        out.push_str(&format!(" colspan=\"{colspan}\""));
    }
    out.push('>');
    render_children(node, out);
    out.push_str("</");
    out.push_str(tag);
    out.push('>');
}

fn wrap_tag(tag: &str, node: &AdfNode, out: &mut String) {
    out.push('<');
    out.push_str(tag);
    out.push('>');
    render_children(node, out);
    out.push_str("</");
    out.push_str(tag);
    out.push('>');
}

fn render_children(node: &AdfNode, out: &mut String) {
    for child in &node.children {
        render_node(child, out);
    }
}

/// Escape HTML special characters: & < > " '
fn escape_html(text: &str, out: &mut String) {
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(ch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adf_node::parse_adf;

    fn html(json: &str) -> String {
        let node = parse_adf(json).unwrap();
        render_html(&node)
    }

    #[test]
    fn simple_paragraph() {
        let result = html(r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Hello"}]}]}"#);
        assert_eq!(result, "<p>Hello</p>");
    }

    #[test]
    fn bold_text() {
        let result = html(r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"bold","marks":[{"type":"strong"}]}]}]}"#);
        assert_eq!(result, "<p><strong>bold</strong></p>");
    }

    #[test]
    fn html_escaping() {
        let result = html(r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"<script>alert('xss')&\"</script>"}]}]}"#);
        assert!(result.contains("&lt;script&gt;"));
        assert!(result.contains("&amp;"));
        assert!(result.contains("&quot;"));
        assert!(result.contains("&#x27;"));
    }

    #[test]
    fn heading() {
        let result = html(r#"{"type":"heading","attrs":{"level":2},"content":[{"type":"text","text":"Title"}]}"#);
        assert_eq!(result, "<h2>Title</h2>");
    }

    #[test]
    fn code_block() {
        let result = html(r#"{"type":"codeBlock","attrs":{"language":"python"},"content":[{"type":"text","text":"print('hi')"}]}"#);
        assert_eq!(result, "<pre><code class=\"language-python\">print(&#x27;hi&#x27;)</code></pre>");
    }

    #[test]
    fn link_text() {
        let result = html(r#"{"type":"text","text":"click","marks":[{"type":"link","attrs":{"href":"http://example.com"}}]}"#);
        assert_eq!(result, "<a href=\"http://example.com\">click</a>");
    }

    #[test]
    fn bullet_list() {
        let result = html(r#"{"type":"bulletList","content":[{"type":"listItem","content":[{"type":"paragraph","content":[{"type":"text","text":"A"}]}]}]}"#);
        assert_eq!(result, "<ul><li><p>A</p></li></ul>");
    }
}
