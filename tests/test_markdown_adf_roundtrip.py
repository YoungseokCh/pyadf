"""Tests for ADF -> Markdown -> ADF roundtrip on safe cases."""

import pytest

from pyadf import Document


class TestAdfRoundtripExact:
    @pytest.mark.parametrize(
        ("adf", "kwargs"),
        [
            (
                {
                    "type": "doc",
                    "content": [
                        {"type": "paragraph", "content": [{"type": "text", "text": "Hello"}]},
                    ],
                },
                {},
            ),
            (
                {
                    "type": "doc",
                    "content": [
                        {
                            "type": "heading",
                            "attrs": {"level": 2},
                            "content": [{"type": "text", "text": "Title"}],
                        }
                    ],
                },
                {},
            ),
            (
                {
                    "type": "doc",
                    "content": [
                        {
                            "type": "paragraph",
                            "content": [
                                {"type": "text", "text": "Hello, "},
                                {
                                    "type": "text",
                                    "text": "world!",
                                    "marks": [{"type": "strong"}],
                                },
                            ],
                        }
                    ],
                },
                {},
            ),
            (
                {
                    "type": "doc",
                    "content": [
                        {
                            "type": "paragraph",
                            "content": [
                                {"type": "text", "text": "A"},
                                {"type": "hardBreak"},
                                {"type": "text", "text": "B"},
                            ],
                        }
                    ],
                },
                {},
            ),
            (
                {
                    "type": "doc",
                    "content": [
                        {
                            "type": "bulletList",
                            "content": [
                                {
                                    "type": "listItem",
                                    "content": [
                                        {
                                            "type": "paragraph",
                                            "content": [{"type": "text", "text": "A"}],
                                        }
                                    ],
                                },
                                {
                                    "type": "listItem",
                                    "content": [
                                        {
                                            "type": "paragraph",
                                            "content": [{"type": "text", "text": "B"}],
                                        }
                                    ],
                                },
                            ],
                        }
                    ],
                },
                {},
            ),
            (
                {
                    "type": "doc",
                    "content": [
                        {
                            "type": "orderedList",
                            "content": [
                                {
                                    "type": "listItem",
                                    "content": [
                                        {
                                            "type": "paragraph",
                                            "content": [{"type": "text", "text": "A"}],
                                        }
                                    ],
                                },
                                {
                                    "type": "listItem",
                                    "content": [
                                        {
                                            "type": "paragraph",
                                            "content": [{"type": "text", "text": "B"}],
                                        }
                                    ],
                                },
                            ],
                        }
                    ],
                },
                {},
            ),
            (
                {
                    "type": "doc",
                    "content": [
                        {
                            "type": "blockquote",
                            "content": [
                                {"type": "paragraph", "content": [{"type": "text", "text": "A"}]},
                                {"type": "paragraph", "content": [{"type": "text", "text": "B"}]},
                            ],
                        }
                    ],
                },
                {},
            ),
            (
                {
                    "type": "doc",
                    "content": [
                        {
                            "type": "codeBlock",
                            "attrs": {"language": "python"},
                            "content": [{"type": "text", "text": "print('hello')"}],
                        }
                    ],
                },
                {},
            ),
            (
                {
                    "type": "doc",
                    "content": [
                        {
                            "type": "extension",
                            "attrs": {"extensionKey": "toc"},
                        }
                    ],
                },
                {"on_known_unsupported": "html"},
            ),
        ],
    )
    def test_safe_adf_roundtrip(self, adf, kwargs):
        markdown = Document(adf).to_markdown(**kwargs)
        reparsed = Document.from_markdown(markdown).to_adf()
        assert reparsed == adf


class TestAdfRoundtripMarkdownStable:
    def test_panel_roundtrips_markdown_as_blockquote(self):
        adf = {
            "type": "doc",
            "content": [
                {
                    "type": "panel",
                    "content": [
                        {"type": "paragraph", "content": [{"type": "text", "text": "Info"}]}
                    ],
                }
            ],
        }

        markdown = Document(adf).to_markdown()
        reparsed_markdown = Document.from_markdown(markdown).to_markdown()
        assert reparsed_markdown == markdown
