"""Tests for ADF -> Markdown -> ADF roundtrip on safe cases."""

import pytest

from pyadf import Document


class TestAdfRoundtripExact:
    @pytest.mark.parametrize(
        ("adf", "kwargs"),
        [
            (
                {
                    "version": 1,
                    "type": "doc",
                    "content": [
                        {"type": "paragraph", "content": [{"type": "text", "text": "Hello"}]},
                    ],
                },
                {},
            ),
            (
                {
                    "version": 1,
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
                    "version": 1,
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
                    "version": 1,
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
                    "version": 1,
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
                    "version": 1,
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
                    "version": 1,
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
                    "version": 1,
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
                    "version": 1,
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


class TestTaskAdfAttrs:
    def test_task_attrs_are_preserved_in_to_adf(self):
        adf = {
            "version": 1,
            "type": "doc",
            "content": [
                {
                    "type": "taskList",
                    "attrs": {"localId": "list-1"},
                    "content": [
                        {
                            "type": "taskItem",
                            "attrs": {"localId": "item-1", "state": "DONE"},
                            "content": [
                                {
                                    "type": "paragraph",
                                    "content": [{"type": "text", "text": "Task"}],
                                }
                            ],
                        }
                    ],
                }
            ],
        }

        assert Document(adf).to_adf() == adf

    def test_task_item_state_controls_markdown_checkbox(self):
        adf = {
            "version": 1,
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
                                    "content": [{"type": "text", "text": "Task"}],
                                }
                            ],
                        }
                    ],
                }
            ],
        }

        assert Document(adf).to_markdown() == "- [x] Task"


class TestAdfRoundtripMarkdownStable:
    def test_panel_roundtrips_markdown_as_blockquote(self):
        adf = {
            "version": 1,
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
