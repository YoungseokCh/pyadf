"""Shared ADF document generators for benchmarks."""

import json


def make_simple_doc(i: int) -> dict:
    """A minimal ADF document (1 paragraph)."""
    return {
        "type": "doc",
        "content": [
            {
                "type": "paragraph",
                "content": [{"type": "text", "text": f"Hello from doc {i}"}],
            }
        ],
    }


def make_rich_doc(i: int) -> dict:
    """A realistic ADF document with heading, status, formatting, links, and list."""
    return {
        "type": "doc",
        "content": [
            {
                "type": "heading",
                "attrs": {"level": 2},
                "content": [{"type": "text", "text": f"Issue PROJ-{i}"}],
            },
            {
                "type": "paragraph",
                "content": [
                    {"type": "text", "text": "Status: "},
                    {"type": "status", "attrs": {"text": "IN PROGRESS", "color": "blue"}},
                ],
            },
            {
                "type": "paragraph",
                "content": [
                    {"type": "text", "text": "This is a "},
                    {"type": "text", "text": "description", "marks": [{"type": "strong"}]},
                    {"type": "text", "text": f" for item {i}. "},
                    {
                        "type": "text",
                        "text": "See docs",
                        "marks": [{"type": "link", "attrs": {"href": "https://example.com"}}],
                    },
                ],
            },
            {
                "type": "bulletList",
                "content": [
                    {
                        "type": "listItem",
                        "content": [
                            {
                                "type": "paragraph",
                                "content": [{"type": "text", "text": "First task"}],
                            }
                        ],
                    },
                    {
                        "type": "listItem",
                        "content": [
                            {
                                "type": "paragraph",
                                "content": [{"type": "text", "text": "Second task"}],
                            }
                        ],
                    },
                ],
            },
        ],
    }


def make_jsonl(docs: list[dict]) -> bytes:
    """Serialize a list of ADF dicts into JSONL bytes."""
    return ("\n".join(json.dumps(d) for d in docs) + "\n").encode()
