# pyadf

![](https://img.shields.io/badge/Python-3776AB?style=flat&logo=python&logoColor=white) ![](https://img.shields.io/pypi/v/pyadf) [![Documentation](https://readthedocs.org/projects/pyadf/badge/?version=latest)](https://pyadf.readthedocs.io)

A high-performance Python library for converting Atlassian document formats to Markdown, powered by Rust via PyO3.

## Features

- **Multi-format input** — ADF, Jira wiki markup, Markdown, and HTML/XHTML
- **Rust-powered** — all parsing and rendering runs in native code via PyO3
- **Streaming JSONL API** for ETL pipelines processing millions of documents
- **Universal `Document` class** — one API for all input formats
- **21 ADF node types** with comprehensive formatting support
- **Type-safe** with comprehensive type hints and Python 3.11+ support
- **Eager validation** — structure errors surface at construction time

## Installation

```bash
pip install pyadf
```

Prebuilt wheels are available for Linux and macOS (x86_64 and aarch64) and Windows (x86_64).

## Quick Start

```python
from pyadf import Document

# From ADF (Atlassian Document Format)
doc = Document({"type": "doc", "content": [
    {"type": "paragraph", "content": [
        {"type": "text", "text": "Hello, "},
        {"type": "text", "text": "world!", "marks": [{"type": "strong"}]}
    ]}
]})
print(doc.to_markdown())  # Hello, **world!**

# From Jira wiki markup
doc = Document("h1. Hello *world*", format="jira")
print(doc.to_markdown())  # # Hello **world**

# From Markdown
doc = Document("# Hello **world**", format="markdown")
print(doc.to_markdown())  # # Hello **world**

# From HTML
doc = Document("<h1>Hello <b>world</b></h1>", format="html")
print(doc.to_markdown())  # # Hello **world**
```

## Supported Formats

| Format | Usage | Parser |
|--------|-------|--------|
| ADF (default) | `Document(adf_dict)` or `Document(json_str)` | serde_json |
| Jira markup | `Document(text, format="jira")` | Custom regex-based |
| Markdown | `Document(text, format="markdown")` | pulldown-cmark |
| HTML/XHTML | `Document(text, format="html")` | html5ever/scraper |

All formats are parsed into an ADF tree internally, then rendered through the same Markdown renderer.

### Markdown to Jira

```python
from pyadf import markdown_to_jira

jira = markdown_to_jira("## Section\n**bold text**")
# "h2. Section\n*bold text*"
```

## Batch JSONL Processing

For ETL pipelines processing large volumes of ADF documents:

```python
from pyadf import convert_jsonl, ConversionError

for result in convert_jsonl("export.jsonl"):
    if isinstance(result, ConversionError):
        print(f"Line {result.line_number}: {result.error}")
    else:
        print(result)
```

## Configuration

```python
from pyadf import Document, MarkdownConfig

config = MarkdownConfig(bullet_marker="*", show_links=True)
doc = Document(adf_data)
doc.to_markdown(config)
```

| Option | Values | Default | Description |
|--------|--------|---------|-------------|
| `bullet_marker` | `+`, `-`, `*` | `+` | Character for bullet list items |
| `show_links` | `True`, `False` | `False` | Include link URLs in output |

## Documentation

Full documentation at [pyadf.readthedocs.io](https://pyadf.readthedocs.io).

## Development

```bash
git clone https://github.com/YoungseokCh/pyadf.git
cd pyadf
uv sync
uv run maturin develop
cargo test              # Rust tests
uv run pytest tests/ -v # Python tests
```

## License

MIT License -- see LICENSE file for details.

## Changelog

### 0.5.0 (Current)

- **Multi-format Document** -- `Document(text, format=)` accepts `"adf"`, `"jira"`, `"markdown"`, `"html"`
- **Jira wiki markup** -- bidirectional conversion (Jira to ADF tree, Markdown to Jira markup)
- **Markdown parser** -- pulldown-cmark-based Markdown to ADF tree conversion
- **HTML/XHTML parser** -- html5ever-based HTML to ADF tree conversion (Confluence storage format)
- **New mark types** -- code, strike, underline, superscript, subscript, textColor
- **Read the Docs** documentation

### 0.4.1

- Fix Linux x86_64 wheel builds

### 0.4.0

- Rust core via PyO3 -- 5x faster single-doc, 24x faster batch processing
- New `convert_jsonl()` streaming API for batch JSONL processing
- Build system switched from setuptools to maturin

### 0.3.x

- Configurable bullet markers, link display, emoji, mentions
