//! Compiled regex patterns for Jira markup <-> Markdown conversion.
//!
//! All patterns are lazily compiled on first use via `LazyLock`.

use regex::Regex;
use std::sync::LazyLock;

// --- Jira -> Markdown patterns ---

pub static RE_J2M_BQ: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^bq\.\s?(.*?)$").unwrap());
pub static RE_J2M_BOLD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\*([^*]+)\*").unwrap());
pub static RE_J2M_ITALIC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"_([^_]+)_").unwrap());
pub static RE_J2M_LIST: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^((?:#|-|\+|\*)+) (.*)$").unwrap());
pub static RE_J2M_HEADER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^h([0-6])\.(.*)$").unwrap());
pub static RE_J2M_INLINE_CODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\{([^}]+)\}\}").unwrap());
pub static RE_J2M_CITATION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\?\?((?:.[^?]|[^?].)+)\?\?").unwrap());
pub static RE_J2M_INS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\+([^+]*)\+").unwrap());
pub static RE_J2M_SUP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\^([^^]*)\^").unwrap());
pub static RE_J2M_SUB: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"~([^~]*)~").unwrap());
pub static RE_J2M_STRIKE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"-([^-]*)-").unwrap());
pub static RE_J2M_CODE_BLOCK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)\{code(?::([a-z]+))?\}([\s\S]*?)\{code\}").unwrap());
pub static RE_J2M_NOFORMAT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)\{noformat\}([\s\S]*?)\{noformat\}").unwrap());
pub static RE_J2M_QUOTE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)\{quote\}([\s\S]*)\{quote\}").unwrap());
pub static RE_J2M_IMG_ALT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"!([^|\n\s]+)\|([^\n!]*)alt=([^\n!,]+?)(?:,([^\n!]*))?!").unwrap()
});
pub static RE_J2M_IMG_PARAMS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"!([^|\n\s]+)\|([^\n!]*)!").unwrap());
pub static RE_J2M_IMG_SIMPLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"!([^\n\s!]+)!").unwrap());
pub static RE_J2M_LINK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([^|]+)\|(.+?)\]").unwrap());
pub static RE_J2M_LINK_BARE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[(.+?)\]([^(]+)").unwrap());
pub static RE_J2M_COLOR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)\{color:([^}]+)\}([\s\S]*?)\{color\}").unwrap());

// --- Markdown -> Jira patterns ---

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

// --- Constants ---

pub const HTML_TAG_MAP: &[(&str, &str)] = &[
    ("cite", "??"),
    ("del", "-"),
    ("ins", "+"),
    ("sup", "^"),
    ("sub", "~"),
];
