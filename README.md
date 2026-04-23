# pyadf

![](https://img.shields.io/badge/Python-3776AB?style=flat&logo=python&logoColor=white) ![](https://img.shields.io/pypi/v/pyadf)

A high-performance Python library for converting [Atlassian Document Format (ADF)](https://developer.atlassian.com/cloud/jira/platform/apis/document/structure/) to Markdown.

## Features

- **Rust-powered** — parsing and rendering run in native code via PyO3
- **Streaming JSONL API** for ETL pipelines processing millions of documents
- **Same `Document` class API** — drop-in upgrade for most users (see changelog for breaking changes)
- **Flexible input** — accepts JSON strings, dictionaries, or any ADF node type
- **Comprehensive node support**:
  - Text formatting (bold, italic, links)
  - Headings (h1-h6)
  - Lists (bullet, ordered, task lists)
  - Tables with headers and column spans
  - Code blocks with syntax highlighting
  - Blockquotes and panels
  - Status badges, inline cards, block cards, emoji, mentions
- **Type-safe** with comprehensive type hints and Python 3.11+ support
- **Eager validation** — ADF structure errors surface at construction time, not render time
- **Robust error handling** with detailed, context-aware error messages

## Installation

```bash
pip install pyadf
```

Prebuilt wheels are available for Linux and macOS (x86_64 and aarch64) and Windows (x86_64).

## Usage

### Basic Usage

```python
from pyadf import Document

adf_data = {
    "type": "doc",
    "content": [
        {
            "type": "paragraph",
            "content": [
                {"type": "text", "text": "Hello, "},
                {"type": "text", "text": "world!", "marks": [{"type": "strong"}]}
            ]
        }
    ]
}

doc = Document(adf_data)
print(doc.to_markdown())
# Output: Hello, **world!**
```

### Converting from JSON String

```python
from pyadf import Document

adf_json = '{"type": "doc", "content": [...]}'
doc = Document(adf_json)
markdown = doc.to_markdown()
```

### Converting Individual Nodes

```python
from pyadf import Document

node = {
    "type": "heading",
    "attrs": {"level": 2},
    "content": [{"type": "text", "text": "My Heading"}]
}

doc = Document(node)
print(doc.to_markdown())
# Output: ## My Heading
```

### Batch JSONL Processing

For ETL pipelines processing large volumes of ADF documents:

```python
from pyadf import convert_jsonl, MarkdownConfig

# From a JSONL file (one ADF document per line)
for result in convert_jsonl("export.jsonl"):
    print(result)

# From bytes with custom config
config = MarkdownConfig(bullet_marker="*", show_links=True)
for result in convert_jsonl(jsonl_bytes, config=config, batch_size=10_000):
    print(result)

# Error handling modes
from pyadf import ConversionError

for result in convert_jsonl(data, on_error="include"):
    if isinstance(result, ConversionError):
        print(f"Line {result.line_number}: {result.error}")
    else:
        print(result)
```

`convert_jsonl` accepts:
- **`source`**: file path (`str`), raw bytes, or a binary file-like object
- **`config`**: optional `MarkdownConfig`
- **`on_error`**: `"include"` (default, yields `ConversionError`), `"skip"`, or `"raise"`
- **`batch_size`**: lines per Rust batch (default 10,000)

### Error Handling

```python
from pyadf import Document, InvalidJSONError, UnsupportedNodeTypeError

try:
    doc = Document('invalid json')
except InvalidJSONError as e:
    print(f"Invalid JSON: {e}")

try:
    doc = Document({"type": "unsupported_type"})
except UnsupportedNodeTypeError as e:
    print(f"Unsupported node: {e}")

# Known unsupported nodes like "extension" can be skipped, warned on, error, or preserved as HTML at render time
doc = Document({"type": "extension"})
assert doc.to_markdown() == ""

doc = Document(
    {
        "type": "extension",
        "attrs": {"extensionKey": "toc", "extensionType": "com.atlassian.confluence.macro.core"},
    }
)
assert doc.to_markdown(on_known_unsupported="html") == (
    '<div adf="extension" '
    'params=\'{"extensionKey":"toc","extensionType":"com.atlassian.confluence.macro.core"}\'></div>'
)
```

Known unsupported node handling:
- `Document(...).to_markdown()` defaults to `on_known_unsupported="warn"` and emits `UserWarning` while skipping known unsupported nodes such as `extension`
- `Document(...).to_markdown(on_known_unsupported="skip")` silently skips known unsupported nodes
- `Document(...).to_markdown(on_known_unsupported="error")` raises `UnsupportedNodeTypeError`
- `Document(...).to_markdown(on_known_unsupported="html")` preserves known unsupported nodes as invisible HTML fallback elements like `<div adf="extension" params='...'></div>` (or `<span ...></span>` in inline/cell contexts)

The same `on_known_unsupported` option is available on `convert_jsonl(...)`.

### Customizing Markdown Output

```python
from pyadf import Document, MarkdownConfig

doc = Document(adf_data)

# Default bullet marker is -
doc.to_markdown()  # "- Item 1\n- Item 2"

# Use * for bullet lists
config = MarkdownConfig(bullet_marker="*")
doc.to_markdown(config)  # "* Item 1\n* Item 2"

# Links are shown by default
doc.to_markdown()  # [Link text](http://example.com)

# Hide underlying href while keeping link text marked
config = MarkdownConfig(show_links=False)
doc.to_markdown(config)  # [Link text]
```

| Option | Values | Default | Description |
|--------|--------|---------|-------------|
| `bullet_marker` | `+`, `-`, `*` | `-` | Character used for bullet list items |
| `show_links` | `True`, `False` | `True` | Show underlying links in markdown |

## Known Unsupported Nodes

These node types are recognized but not rendered. By default they are skipped:

- `mediaSingle`
- `mediaGroup`
- `mediaInline`
- `expand`
- `rule`
- `media`
- `embedCard`
- `extension`

## Supported ADF Node Types

| ADF Node Type | Markdown Output | Notes |
|---------------|-----------------|-------|
| `doc` | Document root | Top-level container |
| `paragraph` | Plain text with newlines | |
| `text` | Text with optional formatting | Supports bold, italic, links |
| `heading` | `# Heading` (levels 1-6) | |
| `bulletList` | `- Item` | |
| `orderedList` | `1. Item` | |
| `taskList` | `- [ ] Task` | Checkbox tasks |
| `codeBlock` | ` ```language\ncode\n``` ` | Optional language syntax |
| `blockquote` | `> Quote` | |
| `panel` | `> Panel content` | Info/warning/error boxes |
| `table` | Markdown table | Supports headers and colspan |
| `status` | `**[STATUS]**` | Status badges |
| `inlineCard` | `[link]` or code block | Link previews |
| `emoji` | Unicode emoji | |
| `hardBreak` | Line break | |
| `mention` | `@DisplayName` | Jira user mentions |
| `blockCard` | `[link]` or code block | Link previews |

## Exception Types

- `PyADFError` — Base exception for all pyadf errors
- `InvalidJSONError` — Raised when JSON parsing fails
- `InvalidInputError` — Raised when input type is incorrect
- `InvalidADFError` — Raised when ADF structure is invalid
- `MissingFieldError` — Raised when required fields are missing
- `InvalidFieldError` — Raised when field values are invalid
- `UnsupportedNodeTypeError` — Raised when encountering unsupported node types
- `NodeCreationError` — Raised when node creation fails

All exceptions include detailed context about the error location in the ADF tree.

## Development

### Prerequisites

- Python 3.11+
- Rust toolchain (stable)
- [maturin](https://www.maturin.rs/) (`uv tool install maturin`)

### Setup

```bash
git clone https://github.com/YoungseokCh/pyadf.git
cd pyadf
uv sync
uv run maturin develop
```

### Testing

```bash
cargo test              # Rust unit tests
uv run pytest tests/ -v # Python tests
```

### Linting

```bash
# Rust
cargo fmt --check
cargo clippy -- -D warnings

# Python
ruff check src/ tests/ benchmarks/
ruff format --check src/ tests/ benchmarks/
```

## License

MIT License — see LICENSE file for details.

## Changelog

### 0.4.3

- Show link targets by default in markdown output
- Use `-` as the default bullet marker
- Treat `extension` as a known unsupported node instead of failing by default
- Add `on_known_unsupported=error|skip|warn` for known unsupported nodes; unknown node types still error

### 0.4.2

- Add support for blockCard node type

### 0.4.1

- Fix linux x86_64 wheel builds

### 0.4.0

- Rust core via PyO3 — 5x faster single-doc, 24x faster batch processing
- New `convert_jsonl()` streaming API for batch JSONL processing
- New `ConversionError` dataclass for structured batch error handling
- Build system switched from setuptools to maturin
- abi3 stable ABI wheels for Linux, macOS (x86_64 + aarch64) and Windows (x86_64)

**Breaking changes:**

- Removed `set_debug_mode()` and `_logger` module (will be replaced with Rust-native tracing in a future release)
- `nodes` and `_types` modules removed (internal implementation replaced by Rust)

### 0.3.2

- Added support for showing href links in markdown output

### 0.3.1

- Added mention node support

### 0.3.0

- Added emoji node support
- Added configurable bullet markers via `MarkdownConfig`

### 0.1.0

- Class-based API with `Document` class
- Support for common ADF node types
- Type-safe architecture with comprehensive type hints (Python 3.11+)
- Flexible input handling (JSON strings, dictionaries, individual nodes)
