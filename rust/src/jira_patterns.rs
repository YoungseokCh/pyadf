//! Compiled regex patterns for Markdown -> Jira markup conversion.
//!
//! All patterns are lazily compiled on first use via `LazyLock`.

use regex::Regex;
use std::sync::LazyLock;

pub static RE_M2J_CODE_BLOCK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)```(\w*)\n([\s\S]+?)```").unwrap());
pub static RE_M2J_INLINE_CODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"`([^`]+)`").unwrap());
pub static RE_M2J_SETEXT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^(.*?)\n([=-])+$").unwrap());
pub static RE_M2J_ATX_HEADER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^([#]+)(.*?)$").unwrap());
pub static RE_M2J_BOLD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\*\*([^*]+)\*\*").unwrap());
pub static RE_M2J_ITALIC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\*([^*]+)\*").unwrap());
pub static RE_M2J_BULLET_LIST: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^(\s*)- (.*)$").unwrap());
pub static RE_M2J_NUM_LIST: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^(\s+)1\. (.*)$").unwrap());
pub static RE_M2J_COLOR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?s)<span style="color:(#[^"]+)">([\s\S]*?)</span>"#).unwrap()
});
pub static RE_M2J_STRIKE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"~~(.*?)~~").unwrap());
pub static RE_M2J_IMG_NO_ALT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"!\[\]\(([^)\n\s]+)\)").unwrap());
pub static RE_M2J_IMG_ALT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"!\[([^\]\n]+)\]\(([^)\n\s]+)\)").unwrap());
pub static RE_M2J_LINK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap());
pub static RE_M2J_LINK_BARE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<([^>]+)>").unwrap());
pub static RE_M2J_TABLE_SEP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\|[-\s|]+\|$").unwrap());

pub const HTML_TAG_MAP: &[(&str, &str)] = &[
    ("cite", "??"),
    ("del", "-"),
    ("ins", "+"),
    ("sup", "^"),
    ("sub", "~"),
];
