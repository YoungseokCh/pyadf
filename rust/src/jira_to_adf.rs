use regex::Regex;
use std::sync::LazyLock;

use crate::adf_node::{AdfNode, Mark, NodeKind};

/// Parse Jira wiki markup into an ADF node tree.
pub fn parse_jira(input: &str) -> AdfNode {
    if input.is_empty() {
        return doc_node(vec![]);
    }

    let (text, blocks) = extract_block_elements(input);
    let lines: Vec<&str> = text.lines().collect();
    let children = parse_lines(&lines, &blocks);
    doc_node(children)
}

// --- Block element extraction ---

struct ExtractedBlocks {
    code_blocks: Vec<(Option<String>, String)>,
    noformat_blocks: Vec<String>,
    quote_blocks: Vec<String>,
}

static RE_CODE_BLOCK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)\{code(?::([a-z]+))?\}([\s\S]*?)\{code\}").unwrap());
static RE_NOFORMAT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)\{noformat\}([\s\S]*?)\{noformat\}").unwrap());
static RE_QUOTE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)\{quote\}([\s\S]*?)\{quote\}").unwrap());

fn extract_block_elements(input: &str) -> (String, ExtractedBlocks) {
    let mut text = input.to_string();
    let mut blocks = ExtractedBlocks {
        code_blocks: Vec::new(),
        noformat_blocks: Vec::new(),
        quote_blocks: Vec::new(),
    };

    text = extract_regex(&mut blocks.code_blocks, &text, &RE_CODE_BLOCK, "CODE");
    text = extract_noformat(&mut blocks.noformat_blocks, &text);
    text = extract_quote(&mut blocks.quote_blocks, &text);
    (text, blocks)
}

fn extract_regex(
    store: &mut Vec<(Option<String>, String)>,
    text: &str,
    re: &Regex,
    prefix: &str,
) -> String {
    let mut result = text.to_string();
    while let Some(m) = re.find(&result) {
        let caps = re.captures(&result).unwrap();
        let lang = caps.get(1).map(|c| c.as_str().to_string());
        let body = caps.get(2).map_or("", |c| c.as_str()).to_string();
        let idx = store.len();
        store.push((lang, body));
        let placeholder = format!("\x00{prefix}_{idx}\x00");
        result = format!("{}{}{}", &result[..m.start()], placeholder, &result[m.end()..]);
    }
    result
}

fn extract_noformat(store: &mut Vec<String>, text: &str) -> String {
    let mut result = text.to_string();
    while let Some(m) = RE_NOFORMAT.find(&result) {
        let caps = RE_NOFORMAT.captures(&result).unwrap();
        let body = caps.get(1).map_or("", |c| c.as_str()).to_string();
        let idx = store.len();
        store.push(body);
        let placeholder = format!("\x00NOFORMAT_{idx}\x00");
        result = format!("{}{}{}", &result[..m.start()], placeholder, &result[m.end()..]);
    }
    result
}

fn extract_quote(store: &mut Vec<String>, text: &str) -> String {
    let mut result = text.to_string();
    while let Some(m) = RE_QUOTE.find(&result) {
        let caps = RE_QUOTE.captures(&result).unwrap();
        let body = caps.get(1).map_or("", |c| c.as_str()).to_string();
        let idx = store.len();
        store.push(body);
        let placeholder = format!("\x00QUOTE_{idx}\x00");
        result = format!("{}{}{}", &result[..m.start()], placeholder, &result[m.end()..]);
    }
    result
}

// --- Line classification ---

#[derive(Debug)]
enum LineKind<'a> {
    Header(u8, &'a str),
    BulletItem(#[allow(dead_code)] usize, &'a str),
    OrderedItem(#[allow(dead_code)] usize, &'a str),
    TableRow(&'a str),
    Blockquote(&'a str),
    CodePlaceholder(usize),
    NoformatPlaceholder(usize),
    QuotePlaceholder(usize),
    Blank,
    Paragraph(&'a str),
}

static RE_HEADER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^h([1-6])\.\s+(.*)$").unwrap());
static RE_BULLET: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\*+)\s+(.*)$").unwrap());
static RE_ORDERED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(#+)\s+(.*)$").unwrap());
static RE_BQ_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^bq\.\s+(.*)$").unwrap());
static RE_TABLE_ROW: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\|.*\|$").unwrap());
static RE_CODE_PH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\x00CODE_(\d+)\x00$").unwrap());
static RE_NOFORMAT_PH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\x00NOFORMAT_(\d+)\x00$").unwrap());
static RE_QUOTE_PH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\x00QUOTE_(\d+)\x00$").unwrap());

fn classify_line<'a>(line: &'a str) -> LineKind<'a> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return LineKind::Blank;
    }
    if let Some(caps) = RE_CODE_PH.captures(trimmed) {
        return LineKind::CodePlaceholder(caps[1].parse().unwrap());
    }
    if let Some(caps) = RE_NOFORMAT_PH.captures(trimmed) {
        return LineKind::NoformatPlaceholder(caps[1].parse().unwrap());
    }
    if let Some(caps) = RE_QUOTE_PH.captures(trimmed) {
        return LineKind::QuotePlaceholder(caps[1].parse().unwrap());
    }
    if let Some(caps) = RE_HEADER.captures(trimmed) {
        let level: u8 = caps[1].parse().unwrap_or(1);
        let text = caps.get(2).map_or("", |m| m.as_str());
        return LineKind::Header(level, text);
    }
    if let Some(caps) = RE_BULLET.captures(trimmed) {
        let depth = caps[1].len();
        let text = caps.get(2).map_or("", |m| m.as_str());
        return LineKind::BulletItem(depth, text);
    }
    if let Some(caps) = RE_ORDERED.captures(trimmed) {
        let depth = caps[1].len();
        let text = caps.get(2).map_or("", |m| m.as_str());
        return LineKind::OrderedItem(depth, text);
    }
    if let Some(caps) = RE_BQ_LINE.captures(trimmed) {
        let text = caps.get(1).map_or("", |m| m.as_str());
        return LineKind::Blockquote(text);
    }
    if RE_TABLE_ROW.is_match(trimmed) {
        return LineKind::TableRow(trimmed);
    }
    LineKind::Paragraph(trimmed)
}

// --- Line grouping into ADF nodes ---

fn parse_lines(lines: &[&str], blocks: &ExtractedBlocks) -> Vec<AdfNode> {
    let mut nodes: Vec<AdfNode> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        match classify_line(lines[i]) {
            LineKind::Blank => { i += 1; }
            LineKind::Header(level, text) => {
                nodes.push(heading_node(level, text));
                i += 1;
            }
            LineKind::BulletItem(..) => {
                let (node, consumed) = collect_list_items(lines, i, blocks, true);
                nodes.push(node);
                i += consumed;
            }
            LineKind::OrderedItem(..) => {
                let (node, consumed) = collect_list_items(lines, i, blocks, false);
                nodes.push(node);
                i += consumed;
            }
            LineKind::TableRow(_) => {
                let (node, consumed) = collect_table_rows(lines, i);
                nodes.push(node);
                i += consumed;
            }
            LineKind::Blockquote(text) => {
                nodes.push(blockquote_node(text));
                i += 1;
            }
            LineKind::CodePlaceholder(idx) => {
                if let Some((lang, body)) = blocks.code_blocks.get(idx) {
                    nodes.push(code_block_node(lang.clone(), body));
                }
                i += 1;
            }
            LineKind::NoformatPlaceholder(idx) => {
                if let Some(body) = blocks.noformat_blocks.get(idx) {
                    nodes.push(code_block_node(None, body));
                }
                i += 1;
            }
            LineKind::QuotePlaceholder(idx) => {
                if let Some(body) = blocks.quote_blocks.get(idx) {
                    nodes.push(quote_block_node(body));
                }
                i += 1;
            }
            LineKind::Paragraph(text) => {
                nodes.push(paragraph_node(text));
                i += 1;
            }
        }
    }
    nodes
}

fn collect_list_items(
    lines: &[&str],
    start: usize,
    _blocks: &ExtractedBlocks,
    is_bullet: bool,
) -> (AdfNode, usize) {
    let mut items: Vec<AdfNode> = Vec::new();
    let mut i = start;
    while i < lines.len() {
        let kind = classify_line(lines[i]);
        match &kind {
            LineKind::BulletItem(_, text) if is_bullet => {
                items.push(list_item_node(text));
                i += 1;
            }
            LineKind::OrderedItem(_, text) if !is_bullet => {
                items.push(list_item_node(text));
                i += 1;
            }
            _ => break,
        }
    }
    let list_kind = if is_bullet { NodeKind::BulletList } else { NodeKind::OrderedList };
    (AdfNode { kind: list_kind, children: items }, i - start)
}

fn collect_table_rows(lines: &[&str], start: usize) -> (AdfNode, usize) {
    let mut rows: Vec<AdfNode> = Vec::new();
    let mut i = start;
    while i < lines.len() {
        if let LineKind::TableRow(text) = classify_line(lines[i]) {
            rows.push(parse_table_row(text));
            i += 1;
        } else {
            break;
        }
    }
    (AdfNode { kind: NodeKind::Table, children: rows }, i - start)
}

fn parse_table_row(line: &str) -> AdfNode {
    let is_header = line.contains("||");
    let content = if is_header {
        line.replace("||", "|")
    } else {
        line.to_string()
    };
    let cells: Vec<&str> = content.trim_matches('|').split('|').collect();
    let children: Vec<AdfNode> = cells.iter().map(|cell| {
        let cell_text = cell.trim();
        let kind = if is_header {
            NodeKind::TableHeader { colspan: 1 }
        } else {
            NodeKind::TableCell { colspan: 1 }
        };
        AdfNode {
            kind,
            children: vec![paragraph_node(cell_text)],
        }
    }).collect();
    AdfNode { kind: NodeKind::TableRow, children }
}

// --- Node constructors ---

fn doc_node(children: Vec<AdfNode>) -> AdfNode {
    AdfNode { kind: NodeKind::Doc, children }
}

fn heading_node(level: u8, text: &str) -> AdfNode {
    AdfNode {
        kind: NodeKind::Heading { level },
        children: parse_inline(text),
    }
}

fn paragraph_node(text: &str) -> AdfNode {
    AdfNode {
        kind: NodeKind::Paragraph,
        children: parse_inline(text),
    }
}

fn blockquote_node(text: &str) -> AdfNode {
    AdfNode {
        kind: NodeKind::Blockquote,
        children: vec![paragraph_node(text)],
    }
}

fn quote_block_node(body: &str) -> AdfNode {
    let paragraphs: Vec<AdfNode> = body
        .split('\n')
        .filter(|l| !l.trim().is_empty())
        .map(|l| paragraph_node(l.trim()))
        .collect();
    AdfNode { kind: NodeKind::Blockquote, children: paragraphs }
}

fn code_block_node(language: Option<String>, body: &str) -> AdfNode {
    let trimmed = body.trim_matches('\n');
    AdfNode {
        kind: NodeKind::CodeBlock { language },
        children: vec![text_node(trimmed, vec![])],
    }
}

fn list_item_node(text: &str) -> AdfNode {
    AdfNode {
        kind: NodeKind::ListItem,
        children: vec![paragraph_node(text)],
    }
}

fn text_node(text: &str, marks: Vec<Mark>) -> AdfNode {
    AdfNode {
        kind: NodeKind::Text { text: text.to_string(), marks },
        children: vec![],
    }
}

fn mark(mark_type: &str) -> Mark {
    Mark { mark_type: mark_type.to_string(), href: None, color: None }
}

fn link_mark(href: &str) -> Mark {
    Mark { mark_type: "link".to_string(), href: Some(href.to_string()), color: None }
}

fn color_mark(color: &str) -> Mark {
    Mark { mark_type: "textColor".to_string(), href: None, color: Some(color.to_string()) }
}

// --- Inline markup parsing ---

/// Represents a segment of text with associated marks, before final node creation.
struct Segment {
    text: String,
    marks: Vec<Mark>,
}

static RE_INLINE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"\{\{([^}]+)\}\}",            // 1: inline code
        r"|\{color:([^}]+)\}([\s\S]*?)\{color\}", // 2,3: color
        r"|\[([^|]+)\|([^\]]+)\]",      // 4,5: link [text|url]
        r"|\*([^*]+)\*",                // 6: bold
        r"|_([^_]+)_",                  // 7: italic
        r"|\-([^-]+)\-",               // 8: strike
        r"|\+([^+]+)\+",               // 9: underline
        r"|\^([^^]+)\^",               // 10: superscript
        r"|~([^~]+)~",                 // 11: subscript
        r"|\?\?([^?]+(?:\?[^?]+)*)\?\?", // 12: citation -> em
    )).unwrap()
});

fn parse_inline(text: &str) -> Vec<AdfNode> {
    let mut nodes: Vec<AdfNode> = Vec::new();
    let mut last_end = 0;

    for caps in RE_INLINE.captures_iter(text) {
        let m = caps.get(0).unwrap();
        if m.start() > last_end {
            nodes.push(text_node(&text[last_end..m.start()], vec![]));
        }
        last_end = m.end();

        let segment = match_inline_capture(&caps);
        nodes.push(text_node(&segment.text, segment.marks));
    }

    if last_end < text.len() {
        nodes.push(text_node(&text[last_end..], vec![]));
    }
    if nodes.is_empty() {
        nodes.push(text_node(text, vec![]));
    }
    nodes
}

fn match_inline_capture(caps: &regex::Captures) -> Segment {
    if let Some(m) = caps.get(1) {
        return Segment { text: m.as_str().to_string(), marks: vec![mark("code")] };
    }
    if let (Some(color_val), Some(body)) = (caps.get(2), caps.get(3)) {
        return Segment {
            text: body.as_str().to_string(),
            marks: vec![color_mark(color_val.as_str())],
        };
    }
    if let (Some(link_text), Some(url)) = (caps.get(4), caps.get(5)) {
        return Segment {
            text: link_text.as_str().to_string(),
            marks: vec![link_mark(url.as_str())],
        };
    }
    if let Some(m) = caps.get(6) {
        return Segment { text: m.as_str().to_string(), marks: vec![mark("strong")] };
    }
    if let Some(m) = caps.get(7) {
        return Segment { text: m.as_str().to_string(), marks: vec![mark("em")] };
    }
    if let Some(m) = caps.get(8) {
        return Segment { text: m.as_str().to_string(), marks: vec![mark("strike")] };
    }
    if let Some(m) = caps.get(9) {
        return Segment { text: m.as_str().to_string(), marks: vec![mark("underline")] };
    }
    if let Some(m) = caps.get(10) {
        return Segment { text: m.as_str().to_string(), marks: vec![mark("superscript")] };
    }
    if let Some(m) = caps.get(11) {
        return Segment { text: m.as_str().to_string(), marks: vec![mark("subsup")] };
    }
    if let Some(m) = caps.get(12) {
        return Segment { text: m.as_str().to_string(), marks: vec![mark("em")] };
    }
    Segment { text: String::new(), marks: vec![] }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MarkdownConfig;
    use crate::markdown::render;

    fn jira_to_md(input: &str) -> String {
        render(&parse_jira(input), &MarkdownConfig::default())
    }

    fn jira_to_md_links(input: &str) -> String {
        let cfg = MarkdownConfig::new("+", true).unwrap();
        render(&parse_jira(input), &cfg)
    }

    #[test]
    fn empty_input() {
        assert_eq!(jira_to_md(""), "");
    }

    #[test]
    fn plain_paragraph() {
        assert_eq!(jira_to_md("Hello world"), "Hello world");
    }

    #[test]
    fn headers() {
        assert_eq!(jira_to_md("h1. Title"), "# Title");
        assert_eq!(jira_to_md("h3. Sub"), "### Sub");
    }

    #[test]
    fn bold_italic() {
        assert_eq!(jira_to_md("*bold*"), "**bold**");
        assert_eq!(jira_to_md("_italic_"), "*italic*");
    }

    #[test]
    fn inline_code() {
        assert_eq!(jira_to_md("use {{code}}"), "use `code`");
    }

    #[test]
    fn strikethrough() {
        assert_eq!(jira_to_md("-struck-"), "~~struck~~");
    }

    #[test]
    fn link() {
        assert_eq!(jira_to_md_links("[Click|http://x.com]"), "[Click](http://x.com)");
    }

    #[test]
    fn code_block() {
        let input = "{code:python}\nprint('hi')\n{code}";
        assert_eq!(jira_to_md(input), "```python\nprint('hi')\n```");
    }

    #[test]
    fn noformat_block() {
        let input = "{noformat}\nraw text\n{noformat}";
        assert_eq!(jira_to_md(input), "```\nraw text\n```");
    }

    #[test]
    fn bullet_list() {
        let input = "* item 1\n* item 2";
        assert_eq!(jira_to_md(input), "+ item 1\n+ item 2");
    }

    #[test]
    fn ordered_list() {
        let input = "# first\n# second";
        assert_eq!(jira_to_md(input), "1. first\n2. second");
    }

    #[test]
    fn blockquote_bq() {
        assert_eq!(jira_to_md("bq. quoted"), "> quoted");
    }

    #[test]
    fn table() {
        let input = "||Name||Age||\n|Alice|30|";
        let result = jira_to_md(input);
        assert!(result.contains("| Name | Age |"));
        assert!(result.contains("| --- | --- |"));
        assert!(result.contains("| Alice | 30 |"));
    }

    #[test]
    fn quote_block() {
        let input = "{quote}\nhello\nworld\n{quote}";
        let result = jira_to_md(input);
        assert!(result.contains("> hello"));
        assert!(result.contains("> world"));
    }
}
