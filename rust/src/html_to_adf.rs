use scraper::{Html, Node};

use crate::adf_node::{AdfNode, Mark, NodeKind};
use crate::html_helpers::{
    collapse_whitespace, doc_node, extract_color_from_style, extract_pre_content, mark, text_node,
    wrap_inline_in_paragraph,
};

/// Parse an HTML/XHTML string into an ADF node tree.
pub fn parse_html(input: &str) -> AdfNode {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return doc_node(vec![]);
    }
    let fragment = Html::parse_fragment(trimmed);
    let root = fragment.tree.root();
    let mut builder = HtmlToAdfBuilder { mark_stack: vec![] };
    let children = builder.process_children_of(root.id(), &fragment);
    let top_level = wrap_inline_in_paragraph(children);
    doc_node(top_level)
}

struct HtmlToAdfBuilder {
    mark_stack: Vec<Mark>,
}

impl HtmlToAdfBuilder {
    fn process_children_of(
        &mut self,
        node_id: ego_tree::NodeId,
        html: &Html,
    ) -> Vec<AdfNode> {
        let node_ref = html.tree.get(node_id).unwrap();
        let mut result = Vec::new();
        for child in node_ref.children() {
            result.extend(self.process_node(child.id(), html));
        }
        result
    }

    fn process_node(&mut self, node_id: ego_tree::NodeId, html: &Html) -> Vec<AdfNode> {
        let node_ref = html.tree.get(node_id).unwrap();
        match node_ref.value() {
            Node::Text(text) => self.process_text(text),
            Node::Element(elem) => self.process_element(node_id, elem, html),
            _ => vec![],
        }
    }

    fn process_text(&self, text: &scraper::node::Text) -> Vec<AdfNode> {
        let s = collapse_whitespace(&text.text);
        if s.is_empty() {
            return vec![];
        }
        vec![text_node(&s, self.mark_stack.clone())]
    }

    fn process_element(
        &mut self,
        node_id: ego_tree::NodeId,
        elem: &scraper::node::Element,
        html: &Html,
    ) -> Vec<AdfNode> {
        let tag = elem.name.local.as_ref();
        if tag.contains(':') {
            return vec![]; // Skip Confluence ac:*/ri:* tags
        }
        match tag {
            "p" => vec![self.block_with_children(node_id, html, NodeKind::Paragraph)],
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let level = tag[1..].parse::<u8>().unwrap_or(1);
                vec![self.block_with_children(node_id, html, NodeKind::Heading { level })]
            }
            "strong" | "b" => self.with_mark(mark("strong"), node_id, html),
            "em" | "i" => self.with_mark(mark("em"), node_id, html),
            "code" => self.with_mark(mark("code"), node_id, html),
            "pre" => vec![self.process_pre(node_id, html)],
            "a" => self.process_link(node_id, elem, html),
            "ul" => vec![self.process_list(node_id, html, true)],
            "ol" => vec![self.process_list(node_id, html, false)],
            "li" => vec![self.process_li(node_id, html)],
            "table" => vec![self.process_table(node_id, html)],
            "tr" => vec![self.process_table_row(node_id, html)],
            "th" => vec![self.process_cell(node_id, elem, html, true)],
            "td" => vec![self.process_cell(node_id, elem, html, false)],
            "blockquote" => vec![self.process_blockquote(node_id, html)],
            "br" => vec![AdfNode { kind: NodeKind::HardBreak, children: vec![] }],
            "del" | "s" | "strike" => self.with_mark(mark("strike"), node_id, html),
            "u" | "ins" => self.with_mark(mark("underline"), node_id, html),
            "sup" => self.with_mark(mark("superscript"), node_id, html),
            "sub" => self.with_mark(mark("subsup"), node_id, html),
            "span" => self.process_span(node_id, elem, html),
            "div" | "section" | "article" | "main" | "header" | "footer"
            | "nav" | "aside" | "figure" | "figcaption" | "tbody" | "thead"
            | "tfoot" => self.process_children_of(node_id, html),
            _ => self.process_children_of(node_id, html),
        }
    }

    fn block_with_children(
        &mut self,
        node_id: ego_tree::NodeId,
        html: &Html,
        kind: NodeKind,
    ) -> AdfNode {
        let children = self.process_children_of(node_id, html);
        AdfNode { kind, children }
    }

    fn with_mark(
        &mut self,
        m: Mark,
        node_id: ego_tree::NodeId,
        html: &Html,
    ) -> Vec<AdfNode> {
        self.mark_stack.push(m);
        let result = self.process_children_of(node_id, html);
        self.mark_stack.pop();
        result
    }

    fn process_link(
        &mut self,
        node_id: ego_tree::NodeId,
        elem: &scraper::node::Element,
        html: &Html,
    ) -> Vec<AdfNode> {
        let href = elem.attr("href").unwrap_or("").to_string();
        let link = Mark { mark_type: "link".to_string(), href: Some(href), color: None };
        self.with_mark(link, node_id, html)
    }

    fn process_pre(&mut self, node_id: ego_tree::NodeId, html: &Html) -> AdfNode {
        let node_ref = html.tree.get(node_id).unwrap();
        let (language, text) = extract_pre_content(&node_ref);
        AdfNode {
            kind: NodeKind::CodeBlock { language },
            children: vec![text_node(&text, vec![])],
        }
    }

    fn process_list(
        &mut self,
        node_id: ego_tree::NodeId,
        html: &Html,
        is_bullet: bool,
    ) -> AdfNode {
        let children = self.process_children_of(node_id, html);
        let kind = if is_bullet { NodeKind::BulletList } else { NodeKind::OrderedList };
        AdfNode { kind, children }
    }

    fn process_li(&mut self, node_id: ego_tree::NodeId, html: &Html) -> AdfNode {
        let children = self.process_children_of(node_id, html);
        AdfNode {
            kind: NodeKind::ListItem,
            children: wrap_inline_in_paragraph(children),
        }
    }

    fn process_table(&mut self, node_id: ego_tree::NodeId, html: &Html) -> AdfNode {
        let children = self.process_children_of(node_id, html);
        let rows = children
            .into_iter()
            .filter(|c| matches!(c.kind, NodeKind::TableRow))
            .collect();
        AdfNode { kind: NodeKind::Table, children: rows }
    }

    fn process_table_row(&mut self, node_id: ego_tree::NodeId, html: &Html) -> AdfNode {
        AdfNode {
            kind: NodeKind::TableRow,
            children: self.process_children_of(node_id, html),
        }
    }

    fn process_cell(
        &mut self,
        node_id: ego_tree::NodeId,
        elem: &scraper::node::Element,
        html: &Html,
        is_header: bool,
    ) -> AdfNode {
        let colspan: u32 = elem.attr("colspan")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1)
            .max(1);
        let children = self.process_children_of(node_id, html);
        let kind = if is_header {
            NodeKind::TableHeader { colspan }
        } else {
            NodeKind::TableCell { colspan }
        };
        AdfNode { kind, children: wrap_inline_in_paragraph(children) }
    }

    fn process_blockquote(&mut self, node_id: ego_tree::NodeId, html: &Html) -> AdfNode {
        let children = self.process_children_of(node_id, html);
        AdfNode {
            kind: NodeKind::Blockquote,
            children: wrap_inline_in_paragraph(children),
        }
    }

    fn process_span(
        &mut self,
        node_id: ego_tree::NodeId,
        elem: &scraper::node::Element,
        html: &Html,
    ) -> Vec<AdfNode> {
        if let Some(color) = extract_color_from_style(elem) {
            let m = Mark { mark_type: "textColor".to_string(), href: None, color: Some(color) };
            return self.with_mark(m, node_id, html);
        }
        self.process_children_of(node_id, html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MarkdownConfig;
    use crate::markdown::render;

    fn html_to_md(input: &str) -> String {
        render(&parse_html(input), &MarkdownConfig::default())
    }

    fn html_to_md_links(input: &str) -> String {
        let cfg = MarkdownConfig::new("+", true).unwrap();
        render(&parse_html(input), &cfg)
    }

    #[test]
    fn empty_input() {
        assert_eq!(html_to_md(""), "");
        assert_eq!(html_to_md("   "), "");
    }

    #[test]
    fn simple_paragraph() {
        assert_eq!(html_to_md("<p>Hello</p>"), "Hello");
    }

    #[test]
    fn bold_and_italic() {
        assert_eq!(
            html_to_md("<p><strong>bold</strong> and <em>italic</em></p>"),
            "**bold** and *italic*"
        );
    }

    #[test]
    fn headers() {
        assert_eq!(html_to_md("<h1>Title</h1>"), "# Title");
        assert_eq!(html_to_md("<h3>Sub</h3>"), "### Sub");
        assert_eq!(html_to_md("<h6>Deep</h6>"), "###### Deep");
    }

    #[test]
    fn link() {
        assert_eq!(
            html_to_md_links(r#"<a href="http://example.com">click</a>"#),
            "[click](http://example.com)"
        );
    }

    #[test]
    fn code_block_with_language() {
        let html = r#"<pre><code class="language-python">print('hi')</code></pre>"#;
        assert_eq!(html_to_md(html), "```python\nprint('hi')\n```");
    }

    #[test]
    fn inline_code_mark() {
        let node = parse_html("<p>use <code>foo()</code></p>");
        let code_node = &node.children[0].children[1];
        match &code_node.kind {
            NodeKind::Text { marks, .. } => assert_eq!(marks[0].mark_type, "code"),
            other => panic!("Expected Text, got: {other:?}"),
        }
    }

    #[test]
    fn blockquote() {
        assert_eq!(html_to_md("<blockquote><p>quote</p></blockquote>"), "> quote");
    }

    #[test]
    fn unordered_list() {
        assert_eq!(html_to_md("<ul><li>A</li><li>B</li></ul>"), "+ A\n+ B");
    }

    #[test]
    fn ordered_list() {
        assert_eq!(html_to_md("<ol><li>A</li><li>B</li></ol>"), "1. A\n2. B");
    }

    #[test]
    fn strike_mark() {
        let node = parse_html("<del>struck</del>");
        match &node.children[0].children[0].kind {
            NodeKind::Text { marks, .. } => assert_eq!(marks[0].mark_type, "strike"),
            other => panic!("Expected Text, got: {other:?}"),
        }
    }

    #[test]
    fn underline_mark() {
        let node = parse_html("<u>underlined</u>");
        match &node.children[0].children[0].kind {
            NodeKind::Text { marks, .. } => assert_eq!(marks[0].mark_type, "underline"),
            other => panic!("Expected Text, got: {other:?}"),
        }
    }

    #[test]
    fn superscript_subscript_marks() {
        let node = parse_html("<sup>sup</sup>");
        match &node.children[0].children[0].kind {
            NodeKind::Text { marks, .. } => assert_eq!(marks[0].mark_type, "superscript"),
            other => panic!("Expected Text, got: {other:?}"),
        }
        let node = parse_html("<sub>sub</sub>");
        match &node.children[0].children[0].kind {
            NodeKind::Text { marks, .. } => assert_eq!(marks[0].mark_type, "subsup"),
            other => panic!("Expected Text, got: {other:?}"),
        }
    }

    #[test]
    fn transparent_div() {
        assert_eq!(html_to_md("<div><p>text</p></div>"), "text");
    }

    #[test]
    fn confluence_macro_skipped() {
        let html = r#"<p>before</p><ac:structured-macro ac:name="toc"></ac:structured-macro><p>after</p>"#;
        let result = html_to_md(html);
        assert!(result.contains("before"));
        assert!(result.contains("after"));
    }

    #[test]
    fn br_tag() {
        let result = html_to_md("<p>line1<br>line2</p>");
        assert!(result.contains("line1"));
        assert!(result.contains("line2"));
    }

    #[test]
    fn color_span() {
        let result = html_to_md(r#"<span style="color:red">colored</span>"#);
        assert!(result.contains("colored"));
    }
}
