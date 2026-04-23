"""Compliance-lite tests for parser/renderer policy boundaries."""

from pyadf import Document


class TestMarkdownCompliance:
    def test_blockquote_markdown_maps_to_blockquote_not_panel(self):
        markdown = "> Info"

        assert Document.from_markdown(markdown).to_adf() == {
            "type": "doc",
            "content": [
                {
                    "type": "blockquote",
                    "content": [
                        {
                            "type": "paragraph",
                            "content": [{"type": "text", "text": "Info"}],
                        }
                    ],
                }
            ],
        }

    def test_html_fallback_uses_div_at_root_context(self):
        adf = {
            "type": "doc",
            "content": [{"type": "extension", "attrs": {"extensionKey": "toc"}}],
        }

        assert Document(adf).to_markdown(on_known_unsupported="html") == (
            '<div adf="extension" params=\'{"extensionKey":"toc"}\'></div>'
        )

    def test_html_fallback_uses_span_in_paragraph_context(self):
        adf = {
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {"type": "text", "text": "Before "},
                        {"type": "extension", "attrs": {"extensionKey": "toc"}},
                    ],
                }
            ],
        }

        assert Document(adf).to_markdown(on_known_unsupported="html") == (
            'Before <span adf="extension" params=\'{"extensionKey":"toc"}\'></span>'
        )

    def test_html_fallback_uses_span_in_table_cell_context(self):
        adf = {
            "type": "doc",
            "content": [
                {
                    "type": "table",
                    "content": [
                        {
                            "type": "tableRow",
                            "content": [
                                {
                                    "type": "tableCell",
                                    "content": [
                                        {"type": "paragraph", "content": [{"type": "text", "text": "Before "}]},
                                        {"type": "extension", "attrs": {"extensionKey": "toc"}},
                                    ],
                                }
                            ],
                        }
                    ],
                }
            ],
        }

        assert Document(adf).to_markdown(on_known_unsupported="html") == (
            '| Before <span adf="extension" params=\'{"extensionKey":"toc"}\'></span> |'
        )
