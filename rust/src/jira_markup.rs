use regex::Regex;

use crate::jira_patterns::*;

/// Convert Jira wiki markup to Markdown format.
pub fn jira_to_markdown(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    let mut output = input.to_string();

    // Block quotes: bq. text -> > text
    output = RE_J2M_BQ.replace_all(&output, "> $1\n").to_string();

    // Text formatting: *bold* -> **bold**, _italic_ -> *italic*
    output = RE_J2M_BOLD.replace_all(&output, "**$1**").to_string();
    output = RE_J2M_ITALIC.replace_all(&output, "*$1*").to_string();

    // Multi-level lists: # item, ## item, * item, ** item
    output = RE_J2M_LIST
        .replace_all(&output, |caps: &regex::Captures| {
            convert_jira_list_to_markdown(&caps[1], &caps[2])
        })
        .to_string();

    // Headers: h1. text -> # text
    output = RE_J2M_HEADER
        .replace_all(&output, |caps: &regex::Captures| {
            let level: usize = caps[1].parse().unwrap_or(1);
            let hashes = "#".repeat(level);
            format!("{hashes}{}", &caps[2])
        })
        .to_string();

    // Inline code: {{code}} -> `code`
    output = RE_J2M_INLINE_CODE.replace_all(&output, "`$1`").to_string();

    // Citation: ??text?? -> <cite>text</cite>
    output = RE_J2M_CITATION
        .replace_all(&output, "<cite>$1</cite>")
        .to_string();

    // Inserted text: +text+ -> <ins>text</ins>
    output = RE_J2M_INS.replace_all(&output, "<ins>$1</ins>").to_string();

    // Superscript: ^text^ -> <sup>text</sup>
    output = RE_J2M_SUP.replace_all(&output, "<sup>$1</sup>").to_string();

    // Subscript: ~text~ -> <sub>text</sub>
    output = RE_J2M_SUB.replace_all(&output, "<sub>$1</sub>").to_string();

    // Strikethrough: passthrough (-text- stays as -text-)
    output = RE_J2M_STRIKE.replace_all(&output, "-$1-").to_string();

    // Code blocks: {code:lang}...{code} -> ```lang\n...\n```
    output = RE_J2M_CODE_BLOCK
        .replace_all(&output, "```$1\n$2\n```")
        .to_string();

    // No format: {noformat}...{noformat} -> ```\n...\n```
    output = RE_J2M_NOFORMAT
        .replace_all(&output, "```\n$1\n```")
        .to_string();

    // Quote blocks: {quote}...{quote} -> > lines
    output = RE_J2M_QUOTE
        .replace_all(&output, |caps: &regex::Captures| {
            caps[1]
                .split('\n')
                .map(|line| format!("> {line}"))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .to_string();

    // Images with alt text
    output = RE_J2M_IMG_ALT.replace_all(&output, "![$3]($1)").to_string();

    // Images with other parameters (ignore them)
    output = RE_J2M_IMG_PARAMS.replace_all(&output, "![]($1)").to_string();

    // Images without parameters
    output = RE_J2M_IMG_SIMPLE.replace_all(&output, "![]($1)").to_string();

    // Links: [text|url] -> [text](url)
    output = RE_J2M_LINK.replace_all(&output, "[$1]($2)").to_string();
    output = RE_J2M_LINK_BARE.replace_all(&output, "<$1>$2").to_string();

    // Colored text
    output = RE_J2M_COLOR
        .replace_all(&output, r#"<span style="color:$1">$2</span>"#)
        .to_string();

    // Table headers: ||header|| -> |header| + separator
    convert_jira_table_headers(&output)
}

/// Convert Markdown to Jira wiki markup format.
pub fn markdown_to_jira(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    let mut output = input.to_string();

    // Convert code blocks first (save as Jira format)
    output = RE_M2J_CODE_BLOCK
        .replace_all(&output, |caps: &regex::Captures| {
            let syntax = &caps[1];
            let content = &caps[2];
            if syntax.is_empty() {
                format!("{{code}}{content}{{code}}")
            } else {
                format!("{{code:{syntax}}}{content}{{code}}")
            }
        })
        .to_string();

    // Convert inline code: `code` -> {{code}}
    output = RE_M2J_INLINE_CODE
        .replace_all(&output, "{{$1}}")
        .to_string();

    // Setext headers: text\n=== -> h1. text, text\n--- -> h2. text
    output = RE_M2J_SETEXT
        .replace_all(&output, |caps: &regex::Captures| {
            let level = if caps[2].starts_with('=') { 1 } else { 2 };
            format!("h{level}. {}", &caps[1])
        })
        .to_string();

    // ATX headers: # text -> h1. text
    output = RE_M2J_ATX_HEADER
        .replace_all(&output, |caps: &regex::Captures| {
            let level = caps[1].len();
            format!("h{level}.{}", &caps[2])
        })
        .to_string();

    // Bold (**text** -> *text*) and italic (*text* -> _text_)
    // Use placeholder for bold to prevent italic pass from re-matching
    output = RE_M2J_BOLD
        .replace_all(&output, "\x00JBOLD_S\x00${1}\x00JBOLD_E\x00")
        .to_string();
    output = RE_M2J_ITALIC.replace_all(&output, "_${1}_").to_string();
    output = output.replace("\x00JBOLD_S\x00", "*");
    output = output.replace("\x00JBOLD_E\x00", "*");

    // Multi-level bulleted list: - item -> * item
    output = RE_M2J_BULLET_LIST
        .replace_all(&output, |caps: &regex::Captures| {
            let indent = &caps[1];
            let content = &caps[2];
            if indent.is_empty() {
                format!("* {content}")
            } else {
                let level = indent.len() / 2;
                let stars = "*".repeat(level + 1);
                format!("{stars} {content}")
            }
        })
        .to_string();

    // Multi-level numbered list (indented): "  1. item" -> ## item
    output = RE_M2J_NUM_LIST
        .replace_all(&output, |caps: &regex::Captures| {
            let indent = &caps[1];
            let content = &caps[2];
            let level = indent.len() / 4 + 2;
            let hashes = "#".repeat(level);
            format!("{hashes} {content}")
        })
        .to_string();

    // HTML tags to Jira markup
    for (tag, replacement) in HTML_TAG_MAP.iter() {
        let re = Regex::new(&format!(r"<{tag}>(.*?)</{tag}>")).unwrap();
        output = re
            .replace_all(&output, format!("{replacement}$1{replacement}"))
            .to_string();
    }

    // Colored text
    output = RE_M2J_COLOR
        .replace_all(&output, "{color:$1}$2{color}")
        .to_string();

    // Strikethrough: ~~text~~ -> -text-
    output = RE_M2J_STRIKE.replace_all(&output, "-$1-").to_string();

    // Images without alt text: ![](url) -> !url!
    output = RE_M2J_IMG_NO_ALT.replace_all(&output, "!$1!").to_string();

    // Images with alt text: ![alt](url) -> !url|alt=alt!
    output = RE_M2J_IMG_ALT.replace_all(&output, "!$2|alt=$1!").to_string();

    // Links: [text](url) -> [text|url]
    output = RE_M2J_LINK.replace_all(&output, "[$1|$2]").to_string();
    output = RE_M2J_LINK_BARE.replace_all(&output, "[$1]").to_string();

    // Convert markdown tables to Jira format
    convert_markdown_table_headers(&output)
}

fn convert_jira_list_to_markdown(bullets: &str, content: &str) -> String {
    let indent_level = bullets.len() - 1;
    let indent = "  ".repeat(indent_level);
    let last_char = bullets.chars().last().unwrap_or('*');
    let prefix = if last_char == '#' { "1." } else { "-" };
    format!("{indent}{prefix} {content}")
}

fn convert_jira_table_headers(input: &str) -> String {
    let lines: Vec<&str> = input.split('\n').collect();
    let mut result: Vec<String> = Vec::new();
    for line in &lines {
        if line.contains("||") {
            let converted = line.replace("||", "|");
            let cell_count = converted.matches('|').count().saturating_sub(1);
            result.push(converted);
            if cell_count > 0 {
                let sep = (0..cell_count).map(|_| "---").collect::<Vec<_>>();
                result.push(format!("|{}|", sep.join("|")));
            }
        } else {
            result.push(line.to_string());
        }
    }
    result.join("\n")
}

fn convert_markdown_table_headers(input: &str) -> String {
    let lines: Vec<&str> = input.split('\n').collect();
    let mut result: Vec<String> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if i + 1 < lines.len() && RE_M2J_TABLE_SEP.is_match(lines[i + 1]) {
            result.push(lines[i].replace('|', "||"));
            i += 2; // skip separator line
        } else {
            result.push(lines[i].to_string());
            i += 1;
        }
    }
    result.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn j2m_bold() {
        assert_eq!(jira_to_markdown("*bold*"), "**bold**");
    }

    #[test]
    fn j2m_italic() {
        assert_eq!(jira_to_markdown("_italic_"), "*italic*");
    }

    #[test]
    fn j2m_header() {
        assert_eq!(jira_to_markdown("h1. Title"), "# Title");
        assert_eq!(jira_to_markdown("h3. Title"), "### Title");
    }

    #[test]
    fn j2m_inline_code() {
        assert_eq!(jira_to_markdown("{{code}}"), "`code`");
    }

    #[test]
    fn j2m_link() {
        assert_eq!(
            jira_to_markdown("[Click here|http://example.com]"),
            "[Click here](http://example.com)"
        );
    }

    #[test]
    fn m2j_bold() {
        assert_eq!(markdown_to_jira("**bold**"), "*bold*");
    }

    #[test]
    fn m2j_italic() {
        assert_eq!(markdown_to_jira("*italic*"), "_italic_");
    }

    #[test]
    fn m2j_header() {
        assert_eq!(markdown_to_jira("# Title"), "h1. Title");
    }

    #[test]
    fn m2j_code_block() {
        assert_eq!(
            markdown_to_jira("```python\nprint('hi')\n```"),
            "{code:python}print('hi')\n{code}"
        );
    }

    #[test]
    fn empty_input() {
        assert_eq!(jira_to_markdown(""), "");
        assert_eq!(markdown_to_jira(""), "");
    }
}
