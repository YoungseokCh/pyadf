"""Tests for new mark types in the ADF renderer (code, strike, underline, etc.)."""

from pyadf import Document, MarkdownConfig


def _text_with_marks(text: str, marks: list[dict]) -> dict:
    """Build a single-paragraph ADF doc with one text node carrying the given marks."""
    return {
        "type": "doc",
        "content": [
            {"type": "paragraph", "content": [{"type": "text", "text": text, "marks": marks}]},
        ],
    }


class TestCodeMark:
    def test_inline_code(self):
        adf = _text_with_marks("foo", [{"type": "code"}])
        assert Document(adf).to_markdown() == "`foo`"

    def test_code_suppresses_bold(self):
        """Code mark takes precedence; bold is not applied inside backticks."""
        adf = _text_with_marks("foo", [{"type": "strong"}, {"type": "code"}])
        assert Document(adf).to_markdown() == "`foo`"

    def test_code_suppresses_italic(self):
        adf = _text_with_marks("foo", [{"type": "em"}, {"type": "code"}])
        assert Document(adf).to_markdown() == "`foo`"


class TestStrikeMark:
    def test_strikethrough(self):
        adf = _text_with_marks("deleted", [{"type": "strike"}])
        assert Document(adf).to_markdown() == "~~deleted~~"


class TestUnderlineMark:
    def test_underline(self):
        adf = _text_with_marks("inserted", [{"type": "underline"}])
        assert Document(adf).to_markdown() == "<ins>inserted</ins>"


class TestSuperscriptMark:
    def test_superscript(self):
        adf = _text_with_marks("2", [{"type": "superscript"}])
        assert Document(adf).to_markdown() == "<sup>2</sup>"


class TestSubsupMark:
    def test_subscript(self):
        adf = _text_with_marks("2", [{"type": "subsup"}])
        assert Document(adf).to_markdown() == "<sub>2</sub>"


class TestTextColorMark:
    def test_color_with_value(self):
        adf = _text_with_marks("red", [{"type": "textColor", "attrs": {"color": "#ff0000"}}])
        assert Document(adf).to_markdown() == '<span style="color:#ff0000">red</span>'

    def test_color_without_value(self):
        """textColor mark without color attr renders text unchanged."""
        adf = _text_with_marks("plain", [{"type": "textColor"}])
        assert Document(adf).to_markdown() == "plain"


class TestCombinedMarks:
    def test_bold_and_strike(self):
        adf = _text_with_marks("text", [{"type": "strong"}, {"type": "strike"}])
        assert Document(adf).to_markdown() == "~~**text**~~"

    def test_italic_and_underline(self):
        adf = _text_with_marks("text", [{"type": "em"}, {"type": "underline"}])
        assert Document(adf).to_markdown() == "<ins>*text*</ins>"
