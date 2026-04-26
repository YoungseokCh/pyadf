"""Expanded direct Markdown -> ADF parse tests."""

import pytest

from pyadf import Document


class TestDirectParsePass:
    def test_underscore_emphasis_maps_to_em_mark(self):
        assert Document.from_markdown("_x_").to_adf() == {
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [{"type": "text", "text": "x", "marks": [{"type": "em"}]}],
                }
            ],
        }

    def test_underscore_strong_maps_to_strong_mark(self):
        assert Document.from_markdown("__x__").to_adf() == {
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {"type": "text", "text": "x", "marks": [{"type": "strong"}]}
                    ],
                }
            ],
        }

    def test_heading_with_strong_mark(self):
        assert Document.from_markdown("## **Title**").to_adf() == {
            "type": "doc",
            "content": [
                {
                    "type": "heading",
                    "attrs": {"level": 2},
                    "content": [
                        {"type": "text", "text": "Title", "marks": [{"type": "strong"}]}
                    ],
                }
            ],
        }

    def test_combined_strong_em_link_order(self):
        assert Document.from_markdown("***[x](http://e.com)***").to_adf() == {
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {
                            "type": "text",
                            "text": "x",
                            "marks": [
                                {"type": "strong"},
                                {"type": "em"},
                                {"type": "link", "attrs": {"href": "http://e.com"}},
                            ],
                        }
                    ],
                }
            ],
        }

    def test_autolink_maps_to_link_mark(self):
        assert Document.from_markdown("<http://e.com>").to_adf() == {
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {
                            "type": "text",
                            "text": "http://e.com",
                            "marks": [
                                {"type": "link", "attrs": {"href": "http://e.com"}}
                            ],
                        }
                    ],
                }
            ],
        }

    def test_inline_code_maps_to_code_mark(self):
        assert Document.from_markdown("`code`").to_adf() == {
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {
                            "type": "text",
                            "text": "code",
                            "marks": [{"type": "code"}],
                        }
                    ],
                }
            ],
        }

    def test_linked_inline_code_uses_code_and_link_marks(self):
        assert Document.from_markdown("[`code`](http://e.com)").to_adf() == {
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {
                            "type": "text",
                            "text": "code",
                            "marks": [
                                {"type": "code"},
                                {"type": "link", "attrs": {"href": "http://e.com"}},
                            ],
                        }
                    ],
                }
            ],
        }

    def test_strikethrough_maps_to_strike_mark(self):
        assert Document.from_markdown("~~x~~").to_adf() == {
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {
                            "type": "text",
                            "text": "x",
                            "marks": [{"type": "strike"}],
                        }
                    ],
                }
            ],
        }

    def test_unchecked_task_list_maps_to_todo_task_item(self):
        assert Document.from_markdown("- [ ] task").to_adf() == {
            "type": "doc",
            "content": [
                {
                    "type": "taskList",
                    "content": [
                        {
                            "type": "taskItem",
                            "attrs": {"state": "TODO"},
                            "content": [
                                {
                                    "type": "paragraph",
                                    "content": [{"type": "text", "text": "task"}],
                                }
                            ],
                        }
                    ],
                }
            ],
        }

    def test_checked_task_list_maps_to_done_task_item(self):
        assert Document.from_markdown("- [x] task").to_adf() == {
            "type": "doc",
            "content": [
                {
                    "type": "taskList",
                    "content": [
                        {
                            "type": "taskItem",
                            "attrs": {"state": "DONE"},
                            "content": [
                                {
                                    "type": "paragraph",
                                    "content": [{"type": "text", "text": "task"}],
                                }
                            ],
                        }
                    ],
                }
            ],
        }

    def test_code_block_with_blank_lines(self):
        assert Document.from_markdown("```\na\n\nb\n```").to_adf() == {
            "type": "doc",
            "content": [
                {
                    "type": "codeBlock",
                    "content": [{"type": "text", "text": "a\n\nb"}],
                }
            ],
        }

    def test_code_block_uses_first_info_token_as_language(self):
        assert Document.from_markdown("```python linenos\nprint(1)\n```").to_adf() == {
            "type": "doc",
            "content": [
                {
                    "type": "codeBlock",
                    "attrs": {"language": "python"},
                    "content": [{"type": "text", "text": "print(1)"}],
                }
            ],
        }

    def test_blockquote_with_list(self):
        assert Document.from_markdown("> - A\n> - B").to_adf() == {
            "type": "doc",
            "content": [
                {
                    "type": "blockquote",
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
                }
            ],
        }

    def test_blockquote_with_code_block(self):
        assert Document.from_markdown("> ```\n> x\n> ```").to_adf() == {
            "type": "doc",
            "content": [
                {
                    "type": "blockquote",
                    "content": [
                        {
                            "type": "codeBlock",
                            "content": [{"type": "text", "text": "x"}],
                        }
                    ],
                }
            ],
        }

    def test_html_fallback_unescapes_params(self):
        markdown = (
            '<div adf="extension" '
            'params=\'{"text":"Tom &amp; Jerry","value":"it&#39;s"}\'></div>'
        )
        assert Document.from_markdown(markdown).to_adf() == {
            "type": "doc",
            "content": [
                {
                    "type": "extension",
                    "attrs": {"text": "Tom & Jerry", "value": "it's"},
                }
            ],
        }


class TestDirectParseKnownGaps:
    def test_nested_bullet_list(self):
        assert Document.from_markdown("- A\n  - B").to_adf() == {
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
                                },
                                {
                                    "type": "bulletList",
                                    "content": [
                                        {
                                            "type": "listItem",
                                            "content": [
                                                {
                                                    "type": "paragraph",
                                                    "content": [{"type": "text", "text": "B"}],
                                                }
                                            ],
                                        }
                                    ],
                                },
                            ],
                        }
                    ],
                }
            ],
        }

    def test_multi_paragraph_list_item(self):
        assert Document.from_markdown("- A\n\n  B").to_adf() == {
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
                                },
                                {
                                    "type": "paragraph",
                                    "content": [{"type": "text", "text": "B"}],
                                },
                            ],
                        }
                    ],
                }
            ],
        }

    def test_table_with_inline_marks(self):
        assert Document.from_markdown(
            "| **A** | [B](http://e.com) |\n| --- | --- |\n| C | D |"
        ).to_adf() == {
            "type": "doc",
            "content": [
                {
                    "type": "table",
                    "content": [
                        {
                            "type": "tableRow",
                            "content": [
                                {
                                    "type": "tableHeader",
                                    "content": [
                                        {"type": "text", "text": "A", "marks": [{"type": "strong"}]}
                                    ],
                                },
                                {
                                    "type": "tableHeader",
                                    "content": [
                                        {
                                            "type": "text",
                                            "text": "B",
                                            "marks": [
                                                {"type": "link", "attrs": {"href": "http://e.com"}}
                                            ],
                                        }
                                    ],
                                },
                            ],
                        },
                        {
                            "type": "tableRow",
                            "content": [
                                {"type": "tableCell", "content": [{"type": "text", "text": "C"}]},
                                {"type": "tableCell", "content": [{"type": "text", "text": "D"}]},
                            ],
                        },
                    ],
                }
            ],
        }
