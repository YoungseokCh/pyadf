"""Tests for Document.to_adf(), to_html(), and to_jira() output methods."""

from pyadf import Document

# --- Shared ADF fixtures ---

SIMPLE_DOC = {"type": "doc", "content": [
    {"type": "paragraph", "content": [{"type": "text", "text": "Hello, world!"}]},
]}

BOLD_DOC = {"type": "doc", "content": [
    {"type": "paragraph", "content": [
        {"type": "text", "text": "bold", "marks": [{"type": "strong"}]},
    ]},
]}

ITALIC_DOC = {"type": "doc", "content": [
    {"type": "paragraph", "content": [
        {"type": "text", "text": "italic", "marks": [{"type": "em"}]},
    ]},
]}

HEADING_DOC = {"type": "heading", "attrs": {"level": 2},
               "content": [{"type": "text", "text": "Title"}]}

LINK_DOC = {"type": "doc", "content": [
    {"type": "paragraph", "content": [
        {"type": "text", "text": "click",
         "marks": [{"type": "link", "attrs": {"href": "http://example.com"}}]},
    ]},
]}

BULLET_LIST_DOC = {"type": "bulletList", "content": [
    {"type": "listItem", "content": [
        {"type": "paragraph", "content": [{"type": "text", "text": "A"}]},
    ]},
    {"type": "listItem", "content": [
        {"type": "paragraph", "content": [{"type": "text", "text": "B"}]},
    ]},
]}

ORDERED_LIST_DOC = {"type": "orderedList", "content": [
    {"type": "listItem", "content": [
        {"type": "paragraph", "content": [{"type": "text", "text": "A"}]},
    ]},
    {"type": "listItem", "content": [
        {"type": "paragraph", "content": [{"type": "text", "text": "B"}]},
    ]},
]}

CODE_BLOCK_DOC = {"type": "codeBlock", "attrs": {"language": "python"},
                  "content": [{"type": "text", "text": "print('hi')"}]}

BLOCKQUOTE_DOC = {"type": "blockquote", "content": [
    {"type": "paragraph", "content": [{"type": "text", "text": "Quote"}]},
]}

TABLE_DOC = {"type": "table", "content": [
    {"type": "tableRow", "content": [
        {"type": "tableHeader", "content": [
            {"type": "paragraph", "content": [{"type": "text", "text": "H1"}]}]},
        {"type": "tableHeader", "content": [
            {"type": "paragraph", "content": [{"type": "text", "text": "H2"}]}]},
    ]},
    {"type": "tableRow", "content": [
        {"type": "tableCell", "content": [
            {"type": "paragraph", "content": [{"type": "text", "text": "A"}]}]},
        {"type": "tableCell", "content": [
            {"type": "paragraph", "content": [{"type": "text", "text": "B"}]}]},
    ]},
]}

ESCAPING_DOC = {"type": "doc", "content": [
    {"type": "paragraph", "content": [
        {"type": "text", "text": '<script>alert("xss")&\'</script>'},
    ]},
]}


# --- to_adf() tests ---

class TestToAdf:
    def test_round_trip_simple(self):
        doc = Document(SIMPLE_DOC)
        assert Document(doc.to_adf()).to_markdown() == doc.to_markdown()

    def test_round_trip_preserves_structure(self):
        adf = Document(SIMPLE_DOC).to_adf()
        assert adf["type"] == "doc"
        assert adf["attrs"]["version"] == 1
        assert adf["content"][0]["content"][0]["text"] == "Hello, world!"

    def test_none_returns_none(self):
        assert Document(None).to_adf() is None
        assert Document().to_adf() is None

    def test_bold_marks_preserved(self):
        text_node = Document(BOLD_DOC).to_adf()["content"][0]["content"][0]
        assert text_node["marks"][0]["type"] == "strong"

    def test_link_attrs_preserved(self):
        text_node = Document(LINK_DOC).to_adf()["content"][0]["content"][0]
        assert text_node["marks"][0]["attrs"]["href"] == "http://example.com"

    def test_heading_level_preserved(self):
        assert Document(HEADING_DOC).to_adf()["attrs"]["level"] == 2

    def test_code_block_language_preserved(self):
        assert Document(CODE_BLOCK_DOC).to_adf()["attrs"]["language"] == "python"


# --- to_html() tests ---

class TestToHtml:
    def test_simple_paragraph(self):
        assert Document(SIMPLE_DOC).to_html() == "<p>Hello, world!</p>"

    def test_bold(self):
        assert Document(BOLD_DOC).to_html() == "<p><strong>bold</strong></p>"

    def test_italic(self):
        assert Document(ITALIC_DOC).to_html() == "<p><em>italic</em></p>"

    def test_heading(self):
        assert Document(HEADING_DOC).to_html() == "<h2>Title</h2>"

    def test_bullet_list(self):
        assert Document(BULLET_LIST_DOC).to_html() == "<ul><li><p>A</p></li><li><p>B</p></li></ul>"

    def test_code_block(self):
        result = Document(CODE_BLOCK_DOC).to_html()
        assert 'class="language-python"' in result
        assert "print(&#x27;hi&#x27;)" in result

    def test_link(self):
        assert '<a href="http://example.com">click</a>' in Document(LINK_DOC).to_html()

    def test_blockquote(self):
        assert Document(BLOCKQUOTE_DOC).to_html() == "<blockquote><p>Quote</p></blockquote>"

    def test_table(self):
        result = Document(TABLE_DOC).to_html()
        assert "<table>" in result and "<th>" in result and "<td>" in result

    def test_html_escaping(self):
        result = Document(ESCAPING_DOC).to_html()
        assert "&lt;script&gt;" in result
        assert "&amp;" in result and "&quot;" in result and "&#x27;" in result
        assert "<script>" not in result

    def test_empty_returns_empty_string(self):
        assert Document(None).to_html() == ""
        assert Document().to_html() == ""


# --- to_jira() tests ---

class TestToJira:
    def test_simple_paragraph(self):
        assert Document(SIMPLE_DOC).to_jira() == "Hello, world!"

    def test_bold(self):
        assert Document(BOLD_DOC).to_jira() == "*bold*"

    def test_italic(self):
        assert Document(ITALIC_DOC).to_jira() == "_italic_"

    def test_heading(self):
        assert Document(HEADING_DOC).to_jira() == "h2. Title"

    def test_bullet_list(self):
        assert Document(BULLET_LIST_DOC).to_jira() == "* A\n* B"

    def test_ordered_list(self):
        assert Document(ORDERED_LIST_DOC).to_jira() == "# A\n# B"

    def test_code_block(self):
        assert Document(CODE_BLOCK_DOC).to_jira() == "{code:python}\nprint('hi')\n{code}"

    def test_link(self):
        assert "[click|http://example.com]" in Document(LINK_DOC).to_jira()

    def test_blockquote(self):
        result = Document(BLOCKQUOTE_DOC).to_jira()
        assert "{quote}" in result and "Quote" in result

    def test_table(self):
        result = Document(TABLE_DOC).to_jira()
        assert "||H1||H2||" in result and "|A|B|" in result

    def test_empty_returns_empty_string(self):
        assert Document(None).to_jira() == ""
        assert Document().to_jira() == ""


# --- Cross-format tests ---

class TestCrossFormat:
    def test_same_adf_all_formats(self):
        doc = Document(SIMPLE_DOC)
        assert doc.to_markdown() == "Hello, world!"
        assert doc.to_html() == "<p>Hello, world!</p>"
        assert doc.to_jira() == "Hello, world!"

    def test_bold_all_formats(self):
        doc = Document(BOLD_DOC)
        assert doc.to_markdown() == "**bold**"
        assert doc.to_html() == "<p><strong>bold</strong></p>"
        assert doc.to_jira() == "*bold*"

    def test_adf_round_trip_preserves_markdown(self):
        for fixture in [SIMPLE_DOC, BOLD_DOC, LINK_DOC, BULLET_LIST_DOC, CODE_BLOCK_DOC]:
            doc = Document(fixture)
            doc2 = Document(doc.to_adf())
            assert doc2.to_markdown() == doc.to_markdown()
