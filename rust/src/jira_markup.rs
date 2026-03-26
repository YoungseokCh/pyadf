use crate::jira_patterns::*;

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
    for (re, replacement) in RE_M2J_HTML_TAGS.iter() {
        output = re.replace_all(&output, replacement.as_str()).to_string();
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
        assert_eq!(markdown_to_jira(""), "");
    }
}
