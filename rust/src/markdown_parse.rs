use crate::adf_node::{AdfNode, Mark, NodeKind};
use crate::errors::AdfError;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, LinkType, Options, Parser, Tag, TagEnd};

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
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options
}

struct ParseState {
    stack: Vec<Frame>,
    marks: Vec<Mark>,
    link_href: Option<String>,
    pending_html: Option<PendingHtml>,
    in_table_head: bool,
}

impl ParseState {
    fn new() -> Self {
        Self {
            stack: vec![Frame::new(NodeKind::Doc)],
            marks: Vec::new(),
            link_href: None,
            pending_html: None,
            in_table_head: false,
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
                        marks: self.active_code_marks()?,
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
            Event::TaskListMarker(checked) => self.mark_current_item_as_task(checked),
            Event::Rule => Err(markdown_error("thematic breaks are not supported yet")),
            Event::Html(raw) | Event::InlineHtml(raw) => self.handle_html(raw.as_ref()),
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
            Tag::Strikethrough => {
                self.marks.push(Mark {
                    mark_type: "strike".to_string(),
                    href: None,
                });
                Ok(())
            }
            Tag::Link {
                link_type,
                dest_url,
                ..
            } => {
                reject_unsupported_link_type(link_type)?;
                self.link_href = Some(dest_url.to_string());
                Ok(())
            }
            Tag::Table(_) => self.push_frame(NodeKind::Table),
            Tag::TableHead => {
                self.in_table_head = true;
                self.push_frame(NodeKind::TableRow)
            }
            Tag::TableRow => self.push_frame(NodeKind::TableRow),
            Tag::TableCell => {
                let kind = if self.in_table_head {
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
            TagEnd::TableHead => {
                self.in_table_head = false;
                self.pop_frame()
            }
            TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough => {
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
        self.push_node_or_split_mixed_paragraph(node);
        Ok(())
    }

    fn push_child(&mut self, node: AdfNode) {
        if let Some(frame) = self.stack.last_mut() {
            frame.children.push(node);
        }
    }

    fn push_node_or_split_mixed_paragraph(&mut self, node: AdfNode) {
        if !matches!(node.kind, NodeKind::Paragraph) || !node.children.iter().any(is_block_node) {
            self.push_child(node);
            return;
        }

        let mut inline_children = Vec::new();
        for child in node.children {
            if is_block_node(&child) {
                if !inline_children.is_empty() {
                    self.push_child(AdfNode {
                        kind: NodeKind::Paragraph,
                        children: std::mem::take(&mut inline_children),
                    });
                }
                self.push_child(child);
            } else {
                inline_children.push(child);
            }
        }

        if !inline_children.is_empty() {
            self.push_child(AdfNode {
                kind: NodeKind::Paragraph,
                children: inline_children,
            });
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
            "strike" => 2,
            "link" => 3,
            _ => 3,
        });
        marks
    }

    fn active_code_marks(&self) -> Result<Vec<Mark>, AdfError> {
        if !self.marks.is_empty() {
            return Err(markdown_error(
                "inline code can only be combined with links",
            ));
        }

        let mut marks = vec![Mark {
            mark_type: "code".to_string(),
            href: None,
        }];
        if let Some(href) = &self.link_href {
            marks.push(Mark {
                mark_type: "link".to_string(),
                href: Some(href.clone()),
            });
        }
        Ok(marks)
    }

    fn in_code_block(&self) -> bool {
        self.stack
            .last()
            .map(|frame| matches!(frame.kind, NodeKind::CodeBlock { .. }))
            .unwrap_or(false)
    }

    fn mark_current_item_as_task(&mut self, checked: bool) -> Result<(), AdfError> {
        let item = self
            .stack
            .last_mut()
            .ok_or_else(|| markdown_error("task marker outside list item"))?;
        if !matches!(item.kind, NodeKind::ListItem) {
            return Err(markdown_error("task marker outside list item"));
        }

        item.kind = NodeKind::TaskItem {
            local_id: None,
            state: Some(if checked { "DONE" } else { "TODO" }.to_string()),
        };

        let list = self
            .stack
            .iter_mut()
            .rev()
            .find(|frame| matches!(frame.kind, NodeKind::BulletList))
            .ok_or_else(|| markdown_error("task marker outside bullet list"))?;
        list.kind = NodeKind::TaskList { local_id: None };
        Ok(())
    }

    fn finish(mut self) -> Result<AdfNode, AdfError> {
        if self.pending_html.is_some() {
            return Err(markdown_error("unclosed HTML fallback element"));
        }

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
            NodeKind::ListItem | NodeKind::TaskItem { .. } => normalize_block_children(children),
            _ => children,
        };

        AdfNode { kind, children }
    }
}

fn normalize_block_children(children: Vec<AdfNode>) -> Vec<AdfNode> {
    let mut normalized = Vec::new();
    let mut inline_children = Vec::new();

    for child in children {
        if is_block_node(&child) {
            if !inline_children.is_empty() {
                normalized.push(AdfNode {
                    kind: NodeKind::Paragraph,
                    children: std::mem::take(&mut inline_children),
                });
            }
            normalized.push(child);
        } else {
            inline_children.push(child);
        }
    }

    if !inline_children.is_empty() {
        normalized.push(AdfNode {
            kind: NodeKind::Paragraph,
            children: inline_children,
        });
    }

    normalized
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

fn reject_unsupported_link_type(link_type: LinkType) -> Result<(), AdfError> {
    match link_type {
        LinkType::Reference
        | LinkType::ReferenceUnknown
        | LinkType::Collapsed
        | LinkType::CollapsedUnknown
        | LinkType::Shortcut
        | LinkType::ShortcutUnknown => Err(markdown_error(
            "reference-style links are not supported yet",
        )),
        _ => Ok(()),
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
    let params = parse_params_attr(raw)?;

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
    let params = parse_params_attr(raw)?;

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

fn parse_params_attr(raw: &str) -> Option<Option<String>> {
    let Some(params) = extract_attr(raw, "params", '\'').map(html_unescape_attr) else {
        return Some(None);
    };

    if is_valid_json_object(&params) {
        Some(Some(params))
    } else {
        None
    }
}

fn is_valid_json_object(raw: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(raw)
        .ok()
        .and_then(|value| value.as_object().cloned())
        .is_some()
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
            | NodeKind::TaskList { .. }
            | NodeKind::ListItem
            | NodeKind::TaskItem { .. }
            | NodeKind::Panel
            | NodeKind::Blockquote
            | NodeKind::Table
            | NodeKind::TableRow
            | NodeKind::TableHeader { .. }
            | NodeKind::TableCell { .. }
            | NodeKind::CodeBlock { .. }
    )
}
