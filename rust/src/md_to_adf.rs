use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::adf_node::{AdfNode, Mark, NodeKind};
use crate::md_inline;
use crate::node_builders;

/// Parse a Markdown string into an ADF `Doc` node tree.
pub fn parse_markdown(input: &str) -> AdfNode {
    let options = Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TABLES
        | Options::ENABLE_TASKLISTS;
    let parser = Parser::new_ext(input, options);
    let mut builder = AdfBuilder::new();

    for event in parser {
        builder.process_event(event);
    }

    builder.finish()
}

/// Stack-based ADF tree builder driven by pulldown-cmark events.
struct AdfBuilder {
    stack: Vec<BuilderFrame>,
    /// Whether the current row is inside a table header.
    in_table_head: bool,
}

struct BuilderFrame {
    kind: NodeKind,
    children: Vec<AdfNode>,
    /// Accumulated inline marks for text nodes inside this frame.
    marks: Vec<Mark>,
}

impl AdfBuilder {
    fn new() -> Self {
        let root = BuilderFrame {
            kind: NodeKind::Doc,
            children: Vec::new(),
            marks: Vec::new(),
        };
        Self {
            stack: vec![root],
            in_table_head: false,
        }
    }

    fn process_event(&mut self, event: Event) {
        match event {
            Event::Start(tag) => self.handle_start(tag),
            Event::End(tag) => self.handle_end(tag),
            Event::Text(text) => self.add_text_node(&text),
            Event::Code(text) => self.add_code_text(&text),
            Event::SoftBreak => self.add_text_node(" "),
            Event::HardBreak => self.add_child(AdfNode {
                kind: NodeKind::HardBreak,
                children: vec![],
            }),
            Event::InlineHtml(html) => self.handle_inline_html(&html),
            Event::Html(html) => self.handle_inline_html(&html),
            Event::TaskListMarker(_checked) => {
                self.convert_last_item_to_task();
            }
            _ => {}
        }
    }

    fn handle_start(&mut self, tag: Tag) {
        let kind = match tag {
            Tag::Paragraph => NodeKind::Paragraph,
            Tag::Heading { level, .. } => NodeKind::Heading {
                level: heading_to_u8(level),
            },
            Tag::BlockQuote(_) => NodeKind::Blockquote,
            Tag::CodeBlock(cb_kind) => NodeKind::CodeBlock {
                language: extract_language(&cb_kind),
            },
            Tag::List(None) => NodeKind::BulletList,
            Tag::List(Some(_)) => NodeKind::OrderedList,
            Tag::Item => NodeKind::ListItem,
            Tag::Table(_) => NodeKind::Table,
            Tag::TableHead => {
                self.in_table_head = true;
                NodeKind::TableRow
            }
            Tag::TableRow => NodeKind::TableRow,
            Tag::TableCell => {
                if self.in_table_head {
                    NodeKind::TableHeader { colspan: 1 }
                } else {
                    NodeKind::TableCell { colspan: 1 }
                }
            }
            Tag::Emphasis => return self.push_mark(node_builders::mark("em")),
            Tag::Strong => return self.push_mark(node_builders::mark("strong")),
            Tag::Strikethrough => return self.push_mark(node_builders::mark("strike")),
            Tag::Link { dest_url, .. } => {
                return self.push_mark(Mark {
                    mark_type: "link".to_string(),
                    href: Some(dest_url.to_string()),
                    color: None,
                });
            }
            _ => return, // skip unsupported tags
        };
        self.push_frame(kind);
    }

    fn handle_end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough | TagEnd::Link => {
                self.pop_mark();
            }
            TagEnd::TableHead => {
                self.in_table_head = false;
                self.pop_frame();
            }
            TagEnd::Paragraph
            | TagEnd::Heading(_)
            | TagEnd::BlockQuote(_)
            | TagEnd::CodeBlock
            | TagEnd::List(_)
            | TagEnd::Item
            | TagEnd::Table
            | TagEnd::TableRow
            | TagEnd::TableCell => {
                self.pop_frame();
            }
            _ => {}
        }
    }

    fn handle_inline_html(&mut self, html: &str) {
        if let Some(m) = md_inline::parse_html_open_tag(html) {
            self.push_mark(m);
        } else if md_inline::is_html_close_tag(html) {
            self.pop_mark();
        }
        // Ignore other HTML
    }

    fn add_text_node(&mut self, text: &str) {
        let marks = self.current_marks();
        // pulldown-cmark includes a trailing newline in code block text
        let text = if self.is_in_code_block() {
            text.strip_suffix('\n').unwrap_or(text)
        } else {
            text
        };
        let node = AdfNode {
            kind: NodeKind::Text {
                text: text.to_string(),
                marks,
            },
            children: vec![],
        };
        self.add_child(node);
    }

    fn add_code_text(&mut self, text: &str) {
        let mut marks = self.current_marks();
        marks.push(node_builders::mark("code"));
        let node = AdfNode {
            kind: NodeKind::Text {
                text: text.to_string(),
                marks,
            },
            children: vec![],
        };
        self.add_child(node);
    }

    /// Convert the last ListItem pushed to the parent into a TaskItem.
    /// pulldown-cmark emits TaskListMarker right after Start(Item).
    fn convert_last_item_to_task(&mut self) {
        let len = self.stack.len();
        if len >= 2 {
            // Convert the parent list from BulletList to TaskList
            if matches!(self.stack[len - 2].kind, NodeKind::BulletList) {
                self.stack[len - 2].kind = NodeKind::TaskList;
            }
            // Convert current frame from ListItem to TaskItem
            if matches!(self.stack[len - 1].kind, NodeKind::ListItem) {
                self.stack[len - 1].kind = NodeKind::TaskItem;
            }
        }
    }

    fn is_in_code_block(&self) -> bool {
        self.stack
            .iter()
            .any(|f| matches!(f.kind, NodeKind::CodeBlock { .. }))
    }

    fn push_frame(&mut self, kind: NodeKind) {
        self.stack.push(BuilderFrame {
            kind,
            children: Vec::new(),
            marks: Vec::new(),
        });
    }

    fn pop_frame(&mut self) {
        if self.stack.len() <= 1 {
            return;
        }
        let frame = self.stack.pop().unwrap();
        let node = AdfNode {
            kind: frame.kind,
            children: frame.children,
        };
        self.add_child(node);
    }

    fn add_child(&mut self, node: AdfNode) {
        if let Some(frame) = self.stack.last_mut() {
            frame.children.push(node);
        }
    }

    fn push_mark(&mut self, m: Mark) {
        if let Some(frame) = self.stack.last_mut() {
            frame.marks.push(m);
        }
    }

    fn pop_mark(&mut self) {
        if let Some(frame) = self.stack.last_mut() {
            frame.marks.pop();
        }
    }

    /// Collect all active marks from the entire stack.
    fn current_marks(&self) -> Vec<Mark> {
        self.stack
            .iter()
            .flat_map(|f| f.marks.iter())
            .cloned()
            .collect()
    }

    fn finish(mut self) -> AdfNode {
        // Collapse remaining stack frames
        while self.stack.len() > 1 {
            self.pop_frame();
        }
        let root = self.stack.pop().unwrap();
        AdfNode {
            kind: root.kind,
            children: root.children,
        }
    }
}

fn heading_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn extract_language(kind: &CodeBlockKind) -> Option<String> {
    match kind {
        CodeBlockKind::Fenced(lang) if !lang.is_empty() => Some(lang.to_string()),
        _ => None,
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MarkdownConfig;
    use crate::markdown::render;

    fn md_roundtrip(input: &str) -> String {
        let cfg = MarkdownConfig::new("+", true).unwrap();
        let node = parse_markdown(input);
        render(&node, &cfg)
    }

    #[test]
    fn plain_paragraph() {
        assert_eq!(md_roundtrip("Hello world"), "Hello world");
    }

    #[test]
    fn two_paragraphs() {
        assert_eq!(md_roundtrip("A\n\nB"), "A\n\nB");
    }

    #[test]
    fn heading_levels() {
        assert_eq!(md_roundtrip("# H1"), "# H1");
        assert_eq!(md_roundtrip("## H2"), "## H2");
        assert_eq!(md_roundtrip("###### H6"), "###### H6");
    }

    #[test]
    fn bold_italic() {
        assert_eq!(md_roundtrip("**bold**"), "**bold**");
        assert_eq!(md_roundtrip("*italic*"), "*italic*");
        assert_eq!(md_roundtrip("***both***"), "***both***");
    }

    #[test]
    fn inline_code() {
        assert_eq!(md_roundtrip("`code`"), "`code`");
    }

    #[test]
    fn strikethrough() {
        assert_eq!(md_roundtrip("~~struck~~"), "~~struck~~");
    }

    #[test]
    fn link() {
        assert_eq!(
            md_roundtrip("[click](http://example.com)"),
            "[click](http://example.com)"
        );
    }

    #[test]
    fn code_block_with_lang() {
        let input = "```python\nprint('hi')\n```";
        assert_eq!(md_roundtrip(input), "```python\nprint('hi')\n```");
    }

    #[test]
    fn code_block_no_lang() {
        let input = "```\nhello\n```";
        assert_eq!(md_roundtrip(input), "```\nhello\n```");
    }

    #[test]
    fn blockquote() {
        assert_eq!(md_roundtrip("> Quote"), "> Quote");
    }

    #[test]
    fn bullet_list() {
        let input = "- A\n- B";
        assert_eq!(md_roundtrip(input), "+ A\n+ B");
    }

    #[test]
    fn ordered_list() {
        let input = "1. A\n2. B";
        assert_eq!(md_roundtrip(input), "1. A\n2. B");
    }

    #[test]
    fn empty_input() {
        assert_eq!(md_roundtrip(""), "");
    }

    #[test]
    fn hard_break() {
        // Two trailing spaces = hard break in markdown
        let input = "line1  \nline2";
        let result = md_roundtrip(input);
        assert!(result.contains("line1"));
        assert!(result.contains("line2"));
    }
}
