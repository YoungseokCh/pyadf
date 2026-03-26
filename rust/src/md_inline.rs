use regex::Regex;
use std::sync::LazyLock;

use crate::adf_node::Mark;

/// Parse an inline HTML opening tag and return the corresponding ADF mark.
/// Supports: <ins>, <sup>, <sub>, <span style="color:...">.
pub fn parse_html_open_tag(html: &str) -> Option<Mark> {
    let trimmed = html.trim();
    if trimmed.eq_ignore_ascii_case("<ins>") {
        return Some(mark("underline"));
    }
    if trimmed.eq_ignore_ascii_case("<sup>") {
        return Some(mark("superscript"));
    }
    if trimmed.eq_ignore_ascii_case("<sub>") {
        return Some(mark("subsup"));
    }
    if let Some(caps) = RE_SPAN_COLOR.captures(trimmed) {
        let color = caps.get(1).map(|m| m.as_str().to_string());
        return Some(Mark {
            mark_type: "textColor".to_string(),
            href: None,
            color,
        });
    }
    None
}

/// Check if an inline HTML fragment is a closing tag for a known mark.
pub fn is_html_close_tag(html: &str) -> bool {
    let trimmed = html.trim().to_ascii_lowercase();
    matches!(
        trimmed.as_str(),
        "</ins>" | "</sup>" | "</sub>" | "</span>"
    )
}

static RE_SPAN_COLOR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)<span\s+style\s*=\s*"color:\s*([^"]+)"\s*>"#).unwrap()
});

fn mark(mark_type: &str) -> Mark {
    Mark {
        mark_type: mark_type.to_string(),
        href: None,
        color: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ins_tag() {
        let m = parse_html_open_tag("<ins>").unwrap();
        assert_eq!(m.mark_type, "underline");
    }

    #[test]
    fn parse_sup_tag() {
        let m = parse_html_open_tag("<sup>").unwrap();
        assert_eq!(m.mark_type, "superscript");
    }

    #[test]
    fn parse_sub_tag() {
        let m = parse_html_open_tag("<sub>").unwrap();
        assert_eq!(m.mark_type, "subsup");
    }

    #[test]
    fn parse_span_color() {
        let m = parse_html_open_tag(r#"<span style="color:#ff0000">"#).unwrap();
        assert_eq!(m.mark_type, "textColor");
        assert_eq!(m.color.as_deref(), Some("#ff0000"));
    }

    #[test]
    fn unknown_tag_returns_none() {
        assert!(parse_html_open_tag("<div>").is_none());
    }

    #[test]
    fn close_tags_detected() {
        assert!(is_html_close_tag("</ins>"));
        assert!(is_html_close_tag("</sup>"));
        assert!(is_html_close_tag("</sub>"));
        assert!(is_html_close_tag("</span>"));
        assert!(!is_html_close_tag("</div>"));
    }
}
