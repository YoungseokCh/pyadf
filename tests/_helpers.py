"""Shared test fixtures for pyadf tests."""

import json


def make_adf_doc(text: str) -> dict:
    """Create a minimal ADF doc with a single paragraph."""
    return {
        "type": "doc",
        "content": [
            {"type": "paragraph", "content": [{"type": "text", "text": text}]},
        ],
    }


def make_adf_json(text: str) -> str:
    """Create a minimal ADF doc as a JSON string."""
    return json.dumps(make_adf_doc(text))


def make_jsonl(docs: list[str]) -> bytes:
    """Join JSON strings into JSONL bytes."""
    return b"\n".join(line.encode() for line in docs) + b"\n"
