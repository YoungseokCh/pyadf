# pyadf

A Python library for converting Atlassian Document Format (ADF) to Markdown.

## Features

- Convert ADF JSON to clean Markdown
- Support for common ADF node types:
  - Text formatting (bold, italic, links)
  - Headings (h1-h6)
  - Lists (bullet, ordered, task lists)
  - Tables with headers
  - Code blocks with syntax highlighting
  - Blockquotes and panels
  - Status badges
  - Inline cards
- Type-safe with comprehensive type hints
- Extensible architecture with registry pattern
- Detailed error handling

## Installation

```bash
pip install pyadf
```

## Usage

### Basic Usage

```python
from pyadf import adf2md

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

markdown_text = adf2md(adf_data)
print(markdown_text)
# Output: Hello, **world!**
```

### Converting Individual Nodes

```python
from pyadf import adf2md

# Convert a single node
node = {
    "type": "heading",
    "attrs": {"level": 2},
    "content": [
        {"type": "text", "text": "My Heading"}
    ]
}

markdown = adf2md(node)
print(markdown)
# Output: ## My Heading
```

### Converting Lists of Nodes

```python
from pyadf import adf2md

# Convert multiple nodes
nodes = [
    {
        "type": "paragraph",
        "content": [{"type": "text", "text": "First paragraph"}]
    },
    {
        "type": "paragraph",
        "content": [{"type": "text", "text": "Second paragraph"}]
    }
]

markdown = adf2md(nodes)
print(markdown)
# Output:
# First paragraph
#
# Second paragraph
```

### Debug Mode

Enable debug mode for detailed logging:

```python
from pyadf import adf2md, set_debug_mode

set_debug_mode(True)
markdown = adf2md(adf_data)
```

## Supported ADF Node Types

| ADF Node Type | Markdown Output |
|---------------|-----------------|
| `paragraph` | Plain text with newlines |
| `text` | Text with optional formatting |
| `heading` | `# Heading` (levels 1-6) |
| `bulletList` | `+ Item` |
| `orderedList` | `1. Item` |
| `taskList` | `- [ ] Task` |
| `codeBlock` | ` ```language\ncode\n``` ` |
| `blockquote` | `> Quote` |
| `panel` | `> Panel content` |
| `table` | Markdown table |
| `status` | `**[STATUS]**` |
| `inlineCard` | `[link]` or code block |

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

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Changelog

### 0.1.0 (Initial Release)

- Initial implementation of ADF to Markdown converter
- Support for common ADF node types
- Type-safe architecture with comprehensive type hints
- Registry pattern for extensibility
