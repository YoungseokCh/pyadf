Changelog
=========

0.5.0 (Unreleased)
-------------------

- **Jira wiki markup support**: ``Document(text, format="jira")`` parses Jira markup into
  ADF tree for rendering
- **Standalone Jira conversion**: ``markdown_to_jira()`` for Markdown to Jira markup
  convenience functions
- **New mark types**: ``code``, ``strike``, ``underline``, ``superscript``, ``subsup``,
  ``textColor`` marks now render to Markdown
- **Read the Docs** documentation

0.4.1
-----

- Fix Linux x86_64 wheel builds

0.4.0
-----

- Rust core via PyO3 -- 5x faster single-doc, 24x faster batch processing
- New ``convert_jsonl()`` streaming API for batch JSONL processing
- New ``ConversionError`` dataclass for structured batch error handling
- Build system switched from setuptools to maturin
- abi3 stable ABI wheels for Linux, macOS (x86_64 + aarch64) and Windows (x86_64)

**Breaking changes:**

- Removed ``set_debug_mode()`` and ``_logger`` module
- ``nodes`` and ``_types`` modules removed (internal implementation replaced by Rust)

0.3.2
-----

- Added support for showing href links in markdown output

0.3.1
-----

- Added mention node support

0.3.0
-----

- Added emoji node support
- Added configurable bullet markers via ``MarkdownConfig``

0.1.0
-----

- Class-based API with ``Document`` class
- Support for common ADF node types
- Type-safe architecture with comprehensive type hints (Python 3.11+)
