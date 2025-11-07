# pyadf

A Python library for converting Atlassian Document Format (ADF) to Markdown.

## Features

- **Class-based API** for clean, object-oriented usage
- **Flexible input** - accepts JSON strings, dictionaries, or any ADF node type
- **Comprehensive node support**:
  - Text formatting (bold, italic, links)
  - Headings (h1-h6)
  - Lists (bullet, ordered, task lists)
  - Tables with headers and column spans
  - Code blocks with syntax highlighting
  - Blockquotes and panels
  - Status badges
  - Inline cards
- **Type-safe** with comprehensive type hints and Python 3.11+ support
- **Extensible architecture** with registry pattern for custom node types
- **Robust error handling** with detailed, context-aware error messages
- **Debug mode** for troubleshooting and development

## Installation

```bash
pip install pyadf
```

## Usage

### Basic Usage

```python
from pyadf import Document

# Convert ADF document to markdown
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
markdown_text = doc.to_markdown()
print(markdown_text)
# Output: Hello, **world!**
```

### Converting from JSON String

```python
from pyadf import Document

# Convert from JSON string
adf_json = '{"type": "doc", "content": [...]}'
doc = Document(adf_json)
markdown = doc.to_markdown()
```

### Converting Individual Nodes

```python
from pyadf import Document

# Convert a single node (any ADF node type)
node = {
    "type": "heading",
    "attrs": {"level": 2},
    "content": [
        {"type": "text", "text": "My Heading"}
    ]
}

doc = Document(node)
markdown = doc.to_markdown()
print(markdown)
# Output: ## My Heading
```

### Error Handling

The library provides detailed error handling with specific exceptions:

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
```

### Debug Mode

Enable debug mode for detailed logging:

```python
from pyadf import Document, set_debug_mode

set_debug_mode(True)
doc = Document(adf_data)
markdown = doc.to_markdown()
```

## Supported ADF Node Types

| ADF Node Type | Markdown Output | Notes |
|---------------|-----------------|-------|
| `doc` | Document root | Top-level container |
| `paragraph` | Plain text with newlines | |
| `text` | Text with optional formatting | Supports bold, italic, links |
| `heading` | `# Heading` (levels 1-6) | |
| `bulletList` | `+ Item` | |
| `orderedList` | `1. Item` | |
| `taskList` | `- [ ] Task` | Checkbox tasks |
| `codeBlock` | ` ```language\ncode\n``` ` | Optional language syntax |
| `blockquote` | `> Quote` | |
| `panel` | `> Panel content` | Info/warning/error boxes |
| `table` | Markdown table | Supports headers and colspan |
| `status` | `**[STATUS]**` | Status badges |
| `inlineCard` | `[link]` or code block | Link previews |
| `hardBreak` | Line break | |

## Exception Types

The library provides specific exceptions for different error scenarios:

- `PyADFError` - Base exception for all pyadf errors
- `InvalidJSONError` - Raised when JSON parsing fails
- `InvalidInputError` - Raised when input type is incorrect
- `InvalidADFError` - Raised when ADF structure is invalid
- `MissingFieldError` - Raised when required fields are missing
- `InvalidFieldError` - Raised when field values are invalid
- `UnsupportedNodeTypeError` - Raised when encountering unsupported node types
- `NodeCreationError` - Raised when node creation fails

All exceptions include detailed context about the error location in the ADF tree.

## Development

### Setting Up Development Environment

```bash
# Clone the repository
git clone https://github.com/yourusername/pyadf.git
cd pyadf

# Install in development mode
pip install -e .
```

### Running Tests

```bash
pytest
```

## License

MIT License - see LICENSE file for details

## Changelog

### 0.1.0 (Current)

- Class-based API with `Document` class
- Better error handling with detailed exceptions and context
- Support for common ADF node types (doc, paragraph, text, headings, lists, tables, etc.)
- Type-safe architecture with comprehensive type hints (Python 3.11+)
- Registry pattern for extensibility
- Flexible input handling (JSON strings, dictionaries, individual nodes)
- Debug mode for troubleshooting
