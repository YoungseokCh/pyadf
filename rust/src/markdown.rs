use crate::adf_node::{AdfNode, KnownUnsupportedMode, KnownUnsupportedNode, NodeKind};
use crate::config::MarkdownConfig;
use crate::errors::AdfError;

/// Rendering context passed through the tree walk.
struct RenderContext<'a> {
    config: &'a MarkdownConfig,
    is_first: bool,
    is_prev_hard_break: bool,
    parent_kind: Option<ParentKind>,
}

struct RenderState {
    known_unsupported_mode: KnownUnsupportedMode,
    warnings: Vec<KnownUnsupportedNode>,
}

pub struct RenderOutcome {
    pub markdown: String,
    pub warnings: Vec<KnownUnsupportedNode>,
}

/// Lightweight tag for parent context (avoids cloning NodeKind).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParentKind {
    Doc,
    Paragraph,
    BulletList,
    OrderedList,
    TaskList,
    ListItem,
    Heading,
    Panel,
    Blockquote,
    Table,
    TableRow,
    TableCell,
}

/// Render an ADF node tree to markdown with known-unsupported handling.
pub fn render(
    node: &AdfNode,
    config: &MarkdownConfig,
    known_unsupported_mode: KnownUnsupportedMode,
) -> Result<RenderOutcome, AdfError> {
    let mut out = String::new();
    let ctx = RenderContext {
        config,
        is_first: true,
        is_prev_hard_break: false,
        parent_kind: None,
    };
    let mut state = RenderState {
        known_unsupported_mode,
        warnings: Vec::new(),
    };
    render_node(node, &ctx, &mut out, &mut state)?;

    Ok(RenderOutcome {
        markdown: out,
        warnings: state.warnings,
    })
}

fn render_node(
    node: &AdfNode,
    ctx: &RenderContext,
    out: &mut String,
    state: &mut RenderState,
) -> Result<(), AdfError> {
    match &node.kind {
        NodeKind::Doc => render_doc(node, ctx, out, state),
        NodeKind::Paragraph => render_paragraph(node, ctx, out, state),
        NodeKind::Text { text, marks } => {
            render_text(text, marks, ctx, out);
            Ok(())
        }
        NodeKind::HardBreak => {
            render_hard_break(out);
            Ok(())
        }
        NodeKind::BulletList => render_bullet_list(node, ctx, out, state),
        NodeKind::OrderedList => render_ordered_list(node, ctx, out, state),
        NodeKind::TaskList => render_task_list(node, ctx, out, state),
        NodeKind::ListItem | NodeKind::TaskItem => render_list_item(node, ctx, out, state),
        NodeKind::Panel => render_panel(node, ctx, out, state),
        NodeKind::Blockquote => render_blockquote(node, ctx, out, state),
        NodeKind::Table => render_table(node, ctx, out, state),
        NodeKind::TableRow => render_table_row(node, ctx, out, state),
        NodeKind::TableHeader { .. } | NodeKind::TableCell { .. } => {
            render_table_cell(node, ctx, out, state)
        }
        NodeKind::CodeBlock { language } => {
            render_code_block(node, language.as_deref(), ctx, out, state)
        }
        NodeKind::InlineCard { url, data } => {
            render_inline_card(url.as_deref(), data.as_deref(), out);
            Ok(())
        }
        NodeKind::Heading { level } => render_heading(node, *level, ctx, out, state),
        NodeKind::Status { text } => {
            render_status(text, out);
            Ok(())
        }
        NodeKind::Emoji { short_name, text } => {
            render_emoji(short_name, text.as_deref(), out);
            Ok(())
        }
        NodeKind::Mention { text } => {
            render_mention(text.as_deref(), out);
            Ok(())
        }
        NodeKind::BlockCard { url, data } => {
            render_block_card(url.as_deref(), data.as_deref(), out);
            Ok(())
        }
        NodeKind::KnownUnsupported {
            node_type,
            node_path,
            params,
        } => {
            render_known_unsupported(node_type, node_path, params.as_deref(), node, ctx, out, state)
        }
    }
}

fn render_doc(
    node: &AdfNode,
    ctx: &RenderContext,
    out: &mut String,
    state: &mut RenderState,
) -> Result<(), AdfError> {
    let mut parts: Vec<String> = Vec::new();
    for child in &node.children {
        let mut child_out = String::new();
        let child_ctx = RenderContext {
            config: ctx.config,
            is_first: true,
            is_prev_hard_break: false,
            parent_kind: Some(ParentKind::Doc),
        };
        render_node(child, &child_ctx, &mut child_out, state)?;
        if !child_out.is_empty() {
            parts.push(child_out);
        }
    }
    out.push_str(&parts.join("\n\n"));
    Ok(())
}

fn render_paragraph(
    node: &AdfNode,
    ctx: &RenderContext,
    out: &mut String,
    state: &mut RenderState,
) -> Result<(), AdfError> {
    let skip_leading = ctx.parent_kind.is_none()
        || ctx.is_first
        || ctx.is_prev_hard_break
        || ctx.parent_kind == Some(ParentKind::ListItem);

    if !skip_leading {
        out.push('\n');
    }

    render_children(node, ctx, out, state)
}

fn render_text(text: &str, marks: &[crate::adf_node::Mark], ctx: &RenderContext, out: &mut String) {
    // Fast path: ~70% of text nodes have no marks
    if marks.is_empty() {
        out.push_str(text);
        return;
    }

    let is_bold = marks.iter().any(|m| m.mark_type == "strong");
    let is_italic = marks.iter().any(|m| m.mark_type == "em");
    let link_mark = marks.iter().find(|m| m.mark_type == "link");

    let mut formatted = text.to_string();

    if is_bold {
        formatted = apply_formatting(&formatted, "**");
    }
    if is_italic {
        formatted = apply_formatting(&formatted, "*");
    }
    if let Some(mark) = link_mark {
        formatted = format!("[{formatted}]");
        if ctx.config.show_links {
            if let Some(href) = &mark.href {
                if !href.is_empty() {
                    formatted = format!("{formatted}({href})");
                }
            }
        }
    }

    out.push_str(&formatted);
}

fn render_hard_break(out: &mut String) {
    out.push_str("  \n");
}

fn render_bullet_list(
    node: &AdfNode,
    ctx: &RenderContext,
    out: &mut String,
    state: &mut RenderState,
) -> Result<(), AdfError> {
    let marker = &ctx.config.bullet_marker;
    let mut items: Vec<String> = Vec::new();
    for child in &node.children {
        let mut item_out = String::new();
        let child_ctx = child_context(ctx, ParentKind::BulletList, false, false);
        render_node(child, &child_ctx, &mut item_out, state)?;
        items.push(format!("{marker} {item_out}"));
    }
    out.push_str(&items.join("\n"));
    Ok(())
}

fn render_ordered_list(
    node: &AdfNode,
    ctx: &RenderContext,
    out: &mut String,
    state: &mut RenderState,
) -> Result<(), AdfError> {
    let mut items: Vec<String> = Vec::new();
    for (idx, child) in node.children.iter().enumerate() {
        let mut item_out = String::new();
        let child_ctx = child_context(ctx, ParentKind::OrderedList, false, false);
        render_node(child, &child_ctx, &mut item_out, state)?;
        items.push(format!("{}. {item_out}", idx + 1));
    }
    out.push_str(&items.join("\n"));
    Ok(())
}

fn render_task_list(
    node: &AdfNode,
    ctx: &RenderContext,
    out: &mut String,
    state: &mut RenderState,
) -> Result<(), AdfError> {
    let mut items: Vec<String> = Vec::new();
    for child in &node.children {
        let mut item_out = String::new();
        let child_ctx = child_context(ctx, ParentKind::TaskList, false, false);
        render_node(child, &child_ctx, &mut item_out, state)?;
        items.push(format!("- [ ] {item_out}"));
    }
    out.push_str(&items.join("\n"));
    Ok(())
}

fn render_list_item(
    node: &AdfNode,
    ctx: &RenderContext,
    out: &mut String,
    state: &mut RenderState,
) -> Result<(), AdfError> {
    render_children_with_parent(node, ctx, ParentKind::ListItem, out, state)
}

fn render_panel(
    node: &AdfNode,
    ctx: &RenderContext,
    out: &mut String,
    state: &mut RenderState,
) -> Result<(), AdfError> {
    let inner = render_children_as_blocks(node, ctx, state)?;
    prefix_lines(&inner, "> ", out);
    Ok(())
}

fn render_blockquote(
    node: &AdfNode,
    ctx: &RenderContext,
    out: &mut String,
    state: &mut RenderState,
) -> Result<(), AdfError> {
    let inner = render_children_as_blocks(node, ctx, state)?;
    let trimmed = inner.trim();
    prefix_lines(trimmed, "> ", out);
    Ok(())
}

/// Render children as separate blocks joined by `\n\n` (like doc rendering).
/// Used by panel/blockquote where each child paragraph needs a blank-line separator.
fn render_children_as_blocks(
    node: &AdfNode,
    ctx: &RenderContext,
    state: &mut RenderState,
) -> Result<String, AdfError> {
    let mut parts: Vec<String> = Vec::new();
    for child in &node.children {
        let mut child_out = String::new();
        let child_ctx = RenderContext {
            config: ctx.config,
            is_first: true,
            is_prev_hard_break: false,
            parent_kind: parent_kind_of(&node.kind),
        };
        render_node(child, &child_ctx, &mut child_out, state)?;
        if !child_out.is_empty() {
            parts.push(child_out);
        }
    }
    Ok(parts.join("\n\n"))
}

/// Prefix every line with `prefix`. Empty lines between paragraphs get just the prefix (e.g. `>`).
fn prefix_lines(text: &str, prefix: &str, out: &mut String) {
    let mut first = true;
    for line in text.split('\n') {
        if !first {
            out.push('\n');
        }
        first = false;
        if line.is_empty() {
            out.push_str(prefix.trim_end());
        } else {
            out.push_str(prefix);
            out.push_str(line);
        }
    }
}

fn render_table(
    node: &AdfNode,
    ctx: &RenderContext,
    out: &mut String,
    state: &mut RenderState,
) -> Result<(), AdfError> {
    let mut rows: Vec<String> = Vec::new();
    for child in &node.children {
        let mut row_out = String::new();
        let child_ctx = child_context(ctx, ParentKind::Table, false, false);
        render_node(child, &child_ctx, &mut row_out, state)?;
        rows.push(row_out);

        let is_header = child
            .children
            .iter()
            .any(|c| matches!(c.kind, NodeKind::TableHeader { .. }));
        if is_header {
            let col_count = row_column_count(child);
            let sep: Vec<&str> = (0..col_count).map(|_| "---").collect();
            rows.push(format!("| {} |", sep.join(" | ")));
        }
    }
    out.push_str(&rows.join("\n"));
    Ok(())
}

fn row_column_count(row_node: &AdfNode) -> usize {
    let mut count = 0;
    for child in &row_node.children {
        match &child.kind {
            NodeKind::TableHeader { colspan } | NodeKind::TableCell { colspan } => {
                count += *colspan as usize;
            }
            _ => {}
        }
    }
    count
}

fn render_table_row(
    node: &AdfNode,
    ctx: &RenderContext,
    out: &mut String,
    state: &mut RenderState,
) -> Result<(), AdfError> {
    let mut cells: Vec<String> = Vec::new();
    for child in &node.children {
        let mut cell_out = String::new();
        let child_ctx = child_context(ctx, ParentKind::TableRow, false, false);
        render_node(child, &child_ctx, &mut cell_out, state)?;
        cells.push(cell_out);
    }
    out.push_str(&format!("| {} |", cells.join(" | ")));
    Ok(())
}

fn render_table_cell(
    node: &AdfNode,
    ctx: &RenderContext,
    out: &mut String,
    state: &mut RenderState,
) -> Result<(), AdfError> {
    render_children(node, ctx, out, state)
}

fn render_code_block(
    node: &AdfNode,
    language: Option<&str>,
    ctx: &RenderContext,
    out: &mut String,
    state: &mut RenderState,
) -> Result<(), AdfError> {
    let lang = language.unwrap_or("");
    let mut content = String::new();
    render_children(node, ctx, &mut content, state)?;

    if lang.is_empty() {
        out.push_str(&format!("```\n{content}\n```"));
    } else {
        out.push_str(&format!("```{lang}\n{content}\n```"));
    }
    Ok(())
}

fn render_inline_card(url: Option<&str>, data: Option<&str>, out: &mut String) {
    if let Some(url) = url {
        out.push_str(&format!("[{url}]"));
    } else if let Some(data) = data {
        out.push_str(&format!("```\n{data}\n```"));
    } else {
        out.push_str("<broken_inlinecard>");
    }
}

fn render_heading(
    node: &AdfNode,
    level: u8,
    ctx: &RenderContext,
    out: &mut String,
    state: &mut RenderState,
) -> Result<(), AdfError> {
    let prefix: String = "#".repeat(level as usize);

    let mut content = String::new();
    render_children(node, ctx, &mut content, state)?;

    out.push_str(&format!("{prefix} {content}"));
    Ok(())
}

fn render_status(text: &str, out: &mut String) {
    out.push_str(&format!("**[{text}]**"));
}

fn render_emoji(short_name: &str, text: Option<&str>, out: &mut String) {
    if let Some(text) = text {
        out.push_str(text);
    } else {
        out.push_str(short_name);
    }
}

fn render_mention(text: Option<&str>, out: &mut String) {
    out.push_str(text.unwrap_or(""));
}

fn render_block_card(url: Option<&str>, data: Option<&str>, out: &mut String) {
    if let Some(url) = url {
        out.push_str(&format!("[{url}]"));
    } else if let Some(data) = data {
        out.push_str(&format!("```\n{data}\n```"));
    } else {
        out.push_str("<broken_blockcard>");
    }
}

fn render_known_unsupported(
    node_type: &str,
    node_path: &str,
    params: Option<&str>,
    node: &AdfNode,
    ctx: &RenderContext,
    out: &mut String,
    state: &mut RenderState,
) -> Result<(), AdfError> {
    match state.known_unsupported_mode {
        KnownUnsupportedMode::Error => {
            return Err(AdfError::UnsupportedNodeType {
                node_type: node_type.to_string(),
                node_path: Some(node_path.to_string()),
                supported_types: None,
            });
        }
        KnownUnsupportedMode::Warn => {
            state.warnings.push(KnownUnsupportedNode {
                node_type: node_type.to_string(),
                node_path: node_path.to_string(),
            });
            return render_children(node, ctx, out, state);
        }
        KnownUnsupportedMode::Skip => {
            return render_children(node, ctx, out, state);
        }
        KnownUnsupportedMode::Html => {}
    }

    let tag = unsupported_tag(ctx);
    let element = unsupported_element(tag, node_type, params);
    out.push_str(&element);

    if node.children.is_empty() {
        return Ok(());
    }

    if tag == "div" {
        let mut children_out = String::new();
        render_children(node, ctx, &mut children_out, state)?;
        if !children_out.is_empty() {
            out.push_str("\n\n");
            out.push_str(&children_out);
        }
        return Ok(());
    }

    render_children(node, ctx, out, state)
}

fn unsupported_tag(ctx: &RenderContext) -> &'static str {
    match ctx.parent_kind {
        Some(ParentKind::Paragraph | ParentKind::Heading | ParentKind::TableCell) => "span",
        _ => "div",
    }
}

fn unsupported_element(tag: &str, node_type: &str, params: Option<&str>) -> String {
    let mut attrs = format!(" adf=\"{node_type}\"");
    if let Some(params) = params {
        attrs.push_str(" params='");
        attrs.push_str(&escape_html_attr(params));
        attrs.push('\'');
    }
    format!("<{tag}{attrs}></{tag}>")
}

fn escape_html_attr(text: &str) -> String {
    text.replace('&', "&amp;").replace('\'', "&#39;")
}

// --- Helpers ---

fn is_hard_break(node: &AdfNode) -> bool {
    matches!(node.kind, NodeKind::HardBreak)
}

fn render_children(
    node: &AdfNode,
    ctx: &RenderContext,
    out: &mut String,
    state: &mut RenderState,
) -> Result<(), AdfError> {
    for (idx, child) in node.children.iter().enumerate() {
        let prev_hard_break = if idx > 0 {
            is_hard_break(&node.children[idx - 1])
        } else {
            false
        };
        let child_ctx = RenderContext {
            config: ctx.config,
            is_first: idx == 0,
            is_prev_hard_break: prev_hard_break,
            parent_kind: parent_kind_of(&node.kind),
        };
        render_node(child, &child_ctx, out, state)?;
    }
    Ok(())
}

fn render_children_with_parent(
    node: &AdfNode,
    ctx: &RenderContext,
    parent: ParentKind,
    out: &mut String,
    state: &mut RenderState,
) -> Result<(), AdfError> {
    for (idx, child) in node.children.iter().enumerate() {
        let prev_hard_break = if idx > 0 {
            is_hard_break(&node.children[idx - 1])
        } else {
            false
        };
        let child_ctx = RenderContext {
            config: ctx.config,
            is_first: idx == 0,
            is_prev_hard_break: prev_hard_break,
            parent_kind: Some(parent),
        };
        render_node(child, &child_ctx, out, state)?;
    }
    Ok(())
}

fn child_context<'a>(
    ctx: &RenderContext<'a>,
    parent: ParentKind,
    is_first: bool,
    is_prev_hard_break: bool,
) -> RenderContext<'a> {
    RenderContext {
        config: ctx.config,
        is_first,
        is_prev_hard_break,
        parent_kind: Some(parent),
    }
}

fn parent_kind_of(kind: &NodeKind) -> Option<ParentKind> {
    match kind {
        NodeKind::Doc => Some(ParentKind::Doc),
        NodeKind::Paragraph => Some(ParentKind::Paragraph),
        NodeKind::BulletList => Some(ParentKind::BulletList),
        NodeKind::OrderedList => Some(ParentKind::OrderedList),
        NodeKind::TaskList => Some(ParentKind::TaskList),
        NodeKind::ListItem | NodeKind::TaskItem => Some(ParentKind::ListItem),
        NodeKind::Heading { .. } => Some(ParentKind::Heading),
        NodeKind::Panel => Some(ParentKind::Panel),
        NodeKind::Blockquote => Some(ParentKind::Blockquote),
        NodeKind::Table => Some(ParentKind::Table),
        NodeKind::TableRow => Some(ParentKind::TableRow),
        NodeKind::TableCell { .. } | NodeKind::TableHeader { .. } => Some(ParentKind::TableCell),
        _ => None,
    }
}

fn apply_formatting(text: &str, symbols: &str) -> String {
    let trimmed = text.trim_end_matches(' ');
    let trailing_count = text.len() - trimmed.len();
    let spaces: String = " ".repeat(trailing_count);
    format!("{symbols}{trimmed}{symbols}{spaces}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adf_node::{KnownUnsupportedMode, parse_adf};

    fn convert(json: &str) -> String {
        let node = parse_adf(json).unwrap().node;
        render(&node, &MarkdownConfig::default(), KnownUnsupportedMode::Skip)
            .unwrap()
            .markdown
    }

    fn convert_with(json: &str, config: &MarkdownConfig) -> String {
        let node = parse_adf(json).unwrap().node;
        render(&node, config, KnownUnsupportedMode::Skip)
            .unwrap()
            .markdown
    }

    #[test]
    fn simple_paragraph() {
        let json = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Hello, world!"}]}]}"#;
        assert_eq!(convert(json), "Hello, world!");
    }

    #[test]
    fn bold_text() {
        let json = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"bold","marks":[{"type":"strong"}]}]}]}"#;
        assert_eq!(convert(json), "**bold**");
    }

    #[test]
    fn italic_text() {
        let json = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"italic","marks":[{"type":"em"}]}]}]}"#;
        assert_eq!(convert(json), "*italic*");
    }

    #[test]
    fn bold_italic_text() {
        let json = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"both","marks":[{"type":"strong"},{"type":"em"}]}]}]}"#;
        assert_eq!(convert(json), "***both***");
    }

    #[test]
    fn link_text_show_by_default() {
        let json = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"click","marks":[{"type":"link","attrs":{"href":"http://example.com/"}}]}]}]}"#;
        assert_eq!(convert(json), "[click](http://example.com/)");
    }

    #[test]
    fn link_text_hide_when_disabled() {
        let json = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"click","marks":[{"type":"link","attrs":{"href":"http://example.com/"}}]}]}]}"#;
        let config = MarkdownConfig::new("-", false).unwrap();
        assert_eq!(convert_with(json, &config), "[click]");
    }

    #[test]
    fn heading_1() {
        let json =
            r#"{"type":"heading","attrs":{"level":1},"content":[{"type":"text","text":"Title"}]}"#;
        assert_eq!(convert(json), "# Title");
    }

    #[test]
    fn heading_2() {
        let json =
            r#"{"type":"heading","attrs":{"level":2},"content":[{"type":"text","text":"Title"}]}"#;
        assert_eq!(convert(json), "## Title");
    }

    #[test]
    fn heading_6() {
        let json =
            r#"{"type":"heading","attrs":{"level":6},"content":[{"type":"text","text":"Title"}]}"#;
        assert_eq!(convert(json), "###### Title");
    }

    #[test]
    fn bullet_list() {
        let json = r#"{"type":"bulletList","content":[{"type":"listItem","content":[{"type":"paragraph","content":[{"type":"text","text":"A"}]}]},{"type":"listItem","content":[{"type":"paragraph","content":[{"type":"text","text":"B"}]}]}]}"#;
        assert_eq!(convert(json), "- A\n- B");
    }

    #[test]
    fn bullet_list_star() {
        let json = r#"{"type":"bulletList","content":[{"type":"listItem","content":[{"type":"paragraph","content":[{"type":"text","text":"A"}]}]}]}"#;
        let config = MarkdownConfig::new("*", false).unwrap();
        assert_eq!(convert_with(json, &config), "* A");
    }

    #[test]
    fn ordered_list() {
        let json = r#"{"type":"orderedList","content":[{"type":"listItem","content":[{"type":"paragraph","content":[{"type":"text","text":"A"}]}]},{"type":"listItem","content":[{"type":"paragraph","content":[{"type":"text","text":"B"}]}]}]}"#;
        assert_eq!(convert(json), "1. A\n2. B");
    }

    #[test]
    fn task_list() {
        let json = r#"{"type":"taskList","content":[{"type":"taskItem","content":[{"type":"paragraph","content":[{"type":"text","text":"Do it"}]}]}]}"#;
        assert_eq!(convert(json), "- [ ] Do it");
    }

    #[test]
    fn code_block_with_lang() {
        let json = r#"{"type":"codeBlock","attrs":{"language":"python"},"content":[{"type":"text","text":"print('hi')"}]}"#;
        assert_eq!(convert(json), "```python\nprint('hi')\n```");
    }

    #[test]
    fn code_block_no_lang() {
        let json = r#"{"type":"codeBlock","content":[{"type":"text","text":"hello"}]}"#;
        assert_eq!(convert(json), "```\nhello\n```");
    }

    #[test]
    fn blockquote() {
        let json = r#"{"type":"blockquote","content":[{"type":"paragraph","content":[{"type":"text","text":"Quote"}]}]}"#;
        assert_eq!(convert(json), "> Quote");
    }

    #[test]
    fn panel() {
        let json = r#"{"type":"panel","attrs":{"panelType":"info"},"content":[{"type":"paragraph","content":[{"type":"text","text":"Info"}]}]}"#;
        assert_eq!(convert(json), "> Info");
    }

    #[test]
    fn status_badge() {
        let json = r#"{"type":"status","attrs":{"text":"DONE","color":"green"}}"#;
        assert_eq!(convert(json), "**[DONE]**");
    }

    #[test]
    fn emoji_with_text() {
        let json = r#"{"type":"emoji","attrs":{"shortName":":grinning:","text":"😀"}}"#;
        assert_eq!(convert(json), "😀");
    }

    #[test]
    fn emoji_without_text() {
        let json = r#"{"type":"emoji","attrs":{"shortName":":grinning:"}}"#;
        assert_eq!(convert(json), ":grinning:");
    }

    #[test]
    fn mention_with_text() {
        let json = r#"{"type":"mention","attrs":{"id":"123","text":"@Alice"}}"#;
        assert_eq!(convert(json), "@Alice");
    }

    #[test]
    fn mention_without_text() {
        let json = r#"{"type":"mention","attrs":{"id":"123"}}"#;
        assert_eq!(convert(json), "");
    }

    #[test]
    fn inline_card_url() {
        let json = r#"{"type":"inlineCard","attrs":{"url":"http://example.com"}}"#;
        assert_eq!(convert(json), "[http://example.com]");
    }

    #[test]
    fn inline_card_broken() {
        let json = r#"{"type":"inlineCard","attrs":{}}"#;
        assert_eq!(convert(json), "<broken_inlinecard>");
    }

    #[test]
    fn block_card_url() {
        let json = r#"{"type":"blockCard","attrs":{"url":"http://example.com"}}"#;
        assert_eq!(convert(json), "[http://example.com]");
    }

    #[test]
    fn block_card_broken() {
        let json = r#"{"type":"blockCard","attrs":{}}"#;
        assert_eq!(convert(json), "<broken_blockcard>");
    }

    #[test]
    fn full_document() {
        let json = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"First"}]},{"type":"paragraph","content":[{"type":"text","text":"Second"}]}]}"#;
        assert_eq!(convert(json), "First\n\nSecond");
    }

    #[test]
    fn trailing_space_formatting() {
        let result = apply_formatting("bold ", "**");
        assert_eq!(result, "**bold** ");
    }
}
