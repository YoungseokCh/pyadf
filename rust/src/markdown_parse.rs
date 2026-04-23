use crate::adf_node::{AdfNode, Mark, NodeKind};
use crate::errors::AdfError;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

pub fn parse_markdown(markdown: &str) -> Result<AdfNode, AdfError> {
    let parser = Parser::new_ext(markdown, markdown_options());
    let mut state = ParseState::new();

    for event in parser {
        state.handle_event(event)?;
    }

    state.finish()
}

fn markdown_options() -> Options {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options
}

struct ParseState {
    stack: Vec<Frame>,
    marks: Vec<Mark>,
    link_href: Option<String>,
    pending_html: Option<PendingHtml>,
}

impl ParseState {
    fn new() -> Self {
        Self {
            stack: vec![Frame::new(NodeKind::Doc)],
            marks: Vec::new(),
            link_href: None,
            pending_html: None,
        }
    }

    fn handle_event(&mut self, event: Event<'_>) -> Result<(), AdfError> {
        match event {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(text) => {
                let text = if self.in_code_block() {
                    text.strip_suffix('\n').unwrap_or(text.as_ref()).to_string()
                } else {
                    text.to_string()
                };
                self.push_child(AdfNode {
                    kind: NodeKind::Text {
                        text,
                        marks: self.active_marks(),
                    },
                    children: Vec::new(),
                });
                Ok(())
            }
            Event::Code(text) => {
                self.push_child(AdfNode {
                    kind: NodeKind::Text {
                        text: text.to_string(),
                        marks: self.active_marks(),
                    },
                    children: Vec::new(),
                });
                Ok(())
            }
            Event::SoftBreak => {
                self.push_child(AdfNode {
                    kind: NodeKind::Text {
                        text: "\n".to_string(),
                        marks: self.active_marks(),
                    },
                    children: Vec::new(),
                });
                Ok(())
            }
            Event::HardBreak => {
                self.push_child(AdfNode {
                    kind: NodeKind::HardBreak,
                    children: Vec::new(),
                });
                Ok(())
            }
            Event::Rule => Err(markdown_error("thematic breaks are not supported yet")),
            Event::Html(raw) | Event::InlineHtml(raw) => {
                self.handle_html(raw.as_ref())
            }
            _ => Err(markdown_error("unsupported markdown syntax")),
        }
    }

    fn handle_html(&mut self, raw: &str) -> Result<(), AdfError> {
        if let Some(node) = parse_known_unsupported_html(raw) {
            self.push_child(node);
            return Ok(());
        }

        if let Some(pending) = parse_known_unsupported_html_open(raw) {
            self.pending_html = Some(pending);
            return Ok(());
        }

        if let Some(pending) = &self.pending_html {
            if html_close_tag(raw, &pending.tag) {
                let pending = self
                    .pending_html
                    .take()
                    .ok_or_else(|| markdown_error("invalid HTML fallback state"))?;
                self.push_child(AdfNode {
                    kind: NodeKind::KnownUnsupported {
                        node_type: pending.node_type.clone(),
                        node_path: pending.node_type,
                        params: pending.params,
                    },
                    children: Vec::new(),
                });
                return Ok(());
            }
        }

        Err(markdown_error("HTML markdown is not supported yet"))
    }

    fn start(&mut self, tag: Tag<'_>) -> Result<(), AdfError> {
        match tag {
            Tag::Paragraph => self.push_frame(NodeKind::Paragraph),
            Tag::HtmlBlock => Ok(()),
            Tag::Heading { level, .. } => self.push_frame(NodeKind::Heading {
                level: heading_level(level),
            }),
            Tag::BlockQuote(_) => self.push_frame(NodeKind::Blockquote),
            Tag::CodeBlock(kind) => self.push_frame(NodeKind::CodeBlock {
                language: code_block_language(kind),
            }),
            Tag::List(Some(_)) => self.push_frame(NodeKind::OrderedList),
            Tag::List(None) => self.push_frame(NodeKind::BulletList),
            Tag::Item => self.push_frame(NodeKind::ListItem),
            Tag::Emphasis => {
                self.marks.push(Mark {
                    mark_type: "em".to_string(),
                    href: None,
                });
                Ok(())
            }
            Tag::Strong => {
                self.marks.push(Mark {
                    mark_type: "strong".to_string(),
                    href: None,
                });
                Ok(())
            }
            Tag::Link { dest_url, .. } => {
                self.link_href = Some(dest_url.to_string());
                Ok(())
            }
            Tag::Table(_) => self.push_frame(NodeKind::Table),
            Tag::TableHead => Ok(()),
            Tag::TableRow => self.push_frame(NodeKind::TableRow),
            Tag::TableCell => {
                let kind = if self.current_table_row_is_header() {
                    NodeKind::TableHeader { colspan: 1 }
                } else {
                    NodeKind::TableCell { colspan: 1 }
                };
                self.push_frame(kind)
            }
            _ => Err(markdown_error("unsupported markdown syntax")),
        }
    }

    fn end(&mut self, tag: TagEnd) -> Result<(), AdfError> {
        match tag {
            TagEnd::Paragraph
            | TagEnd::Heading(_)
            | TagEnd::BlockQuote(_)
            | TagEnd::CodeBlock
            | TagEnd::List(_)
            | TagEnd::Item
            | TagEnd::Table
            | TagEnd::TableRow
            | TagEnd::TableCell => self.pop_frame(),
            TagEnd::HtmlBlock => Ok(()),
            TagEnd::TableHead => Ok(()),
            TagEnd::Emphasis | TagEnd::Strong => {
                self.marks.pop();
                Ok(())
            }
            TagEnd::Link => {
                self.link_href = None;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn push_frame(&mut self, kind: NodeKind) -> Result<(), AdfError> {
        self.stack.push(Frame::new(kind));
        Ok(())
    }

    fn pop_frame(&mut self) -> Result<(), AdfError> {
        let frame = self
            .stack
            .pop()
            .ok_or_else(|| markdown_error("invalid parser stack state"))?;
        let node = frame.into_node();
        self.push_child(node);
        Ok(())
    }

    fn push_child(&mut self, node: AdfNode) {
        if let Some(frame) = self.stack.last_mut() {
            frame.children.push(node);
        }
    }

    fn active_marks(&self) -> Vec<Mark> {
        let mut marks = self.marks.clone();
        if let Some(href) = &self.link_href {
            marks.push(Mark {
                mark_type: "link".to_string(),
                href: Some(href.clone()),
            });
        }
        marks.sort_by_key(|mark| match mark.mark_type.as_str() {
            "strong" => 0,
            "em" => 1,
            "link" => 2,
            _ => 3,
        });
        marks
    }

    fn in_code_block(&self) -> bool {
        self.stack
            .last()
            .map(|frame| matches!(frame.kind, NodeKind::CodeBlock { .. }))
            .unwrap_or(false)
    }

    fn current_table_row_is_header(&self) -> bool {
        self.stack.iter().rev().any(|frame| matches!(frame.kind, NodeKind::TableRow))
            && self
                .stack
                .iter()
                .rev()
                .find(|frame| matches!(frame.kind, NodeKind::Table))
                .map(|table| table.children.is_empty())
                .unwrap_or(false)
    }

    fn finish(mut self) -> Result<AdfNode, AdfError> {
        if self.stack.len() != 1 {
            return Err(markdown_error("unclosed markdown blocks"));
        }
        let root = self
            .stack
            .pop()
            .ok_or_else(|| markdown_error("missing document root"))?;
        Ok(AdfNode {
            kind: root.kind,
            children: root.children,
        })
    }
}

struct Frame {
    kind: NodeKind,
    children: Vec<AdfNode>,
}

struct PendingHtml {
    tag: String,
    node_type: String,
    params: Option<String>,
}

impl Frame {
    fn new(kind: NodeKind) -> Self {
        Self {
            kind,
            children: Vec::new(),
        }
    }

    fn into_node(self) -> AdfNode {
        let Frame { kind, children } = self;
        let children = match kind {
            NodeKind::ListItem if children.first().is_some_and(|child| !is_block_node(child)) => {
                vec![AdfNode {
                    kind: NodeKind::Paragraph,
                    children,
                }]
            }
            _ => children,
        };

        AdfNode { kind, children }
    }
}

fn heading_level(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn code_block_language(kind: CodeBlockKind<'_>) -> Option<String> {
    match kind {
        CodeBlockKind::Indented => None,
        CodeBlockKind::Fenced(info) => info.split_whitespace().next().map(str::to_string),
    }
}

fn markdown_error(message: &str) -> AdfError {
    AdfError::MarkdownParse {
        message: message.to_string(),
    }
}

fn parse_known_unsupported_html(raw: &str) -> Option<AdfNode> {
    let raw = raw.trim();
    let tag = if raw.starts_with("<div ") {
        "div"
    } else if raw.starts_with("<span ") {
        "span"
    } else {
        return None;
    };

    let close = format!("</{tag}>");
    if !raw.ends_with(&close) {
        return None;
    }

    let adf_type = extract_attr(raw, "adf", '"')?;
    let params = extract_attr(raw, "params", '\'').map(html_unescape_attr);

    Some(AdfNode {
        kind: NodeKind::KnownUnsupported {
            node_type: adf_type.clone(),
            node_path: adf_type,
            params,
        },
        children: Vec::new(),
    })
}

fn parse_known_unsupported_html_open(raw: &str) -> Option<PendingHtml> {
    let raw = raw.trim();
    let tag = if raw.starts_with("<div ") && raw.ends_with('>') {
        "div"
    } else if raw.starts_with("<span ") && raw.ends_with('>') {
        "span"
    } else {
        return None;
    };

    let node_type = extract_attr(raw, "adf", '"')?;
    let params = extract_attr(raw, "params", '\'').map(html_unescape_attr);

    Some(PendingHtml {
        tag: tag.to_string(),
        node_type,
        params,
    })
}

fn extract_attr(raw: &str, name: &str, quote: char) -> Option<String> {
    let pattern = format!("{name}={quote}");
    let start = raw.find(&pattern)? + pattern.len();
    let rest = &raw[start..];
    let end = rest.find(quote)?;
    Some(rest[..end].to_string())
}

fn html_unescape_attr(value: String) -> String {
    value.replace("&amp;", "&").replace("&#39;", "'")
}

fn html_close_tag(raw: &str, tag: &str) -> bool {
    raw.trim() == format!("</{tag}>")
}

fn is_block_node(node: &AdfNode) -> bool {
    matches!(
        node.kind,
        NodeKind::Paragraph
            | NodeKind::Heading { .. }
            | NodeKind::BulletList
            | NodeKind::OrderedList
            | NodeKind::TaskList
            | NodeKind::ListItem
            | NodeKind::TaskItem
            | NodeKind::Panel
            | NodeKind::Blockquote
            | NodeKind::Table
            | NodeKind::TableRow
            | NodeKind::TableHeader { .. }
            | NodeKind::TableCell { .. }
            | NodeKind::CodeBlock { .. }
    )
}
