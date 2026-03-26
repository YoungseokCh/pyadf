pyadf
=====

A high-performance Python library for converting Atlassian document formats to Markdown,
powered by a Rust core via PyO3.

.. code-block:: python

   from pyadf import Document

   # From ADF (Atlassian Document Format)
   doc = Document({"type": "doc", "content": [...]})
   print(doc.to_markdown())

   # From Jira wiki markup
   doc = Document("h1. Hello *world*", format="jira")

   # From Markdown
   doc = Document("# Hello **world**", format="markdown")

   # From HTML/XHTML (Confluence storage format)
   doc = Document("<h1>Hello <b>world</b></h1>", format="html")

Features
--------

- **Rust-powered** parsing and rendering via PyO3 (5x faster single-doc, 24x faster batch)
- **Multiple input formats** -- ADF, Jira wiki markup, Markdown, HTML/XHTML
- **Streaming JSONL API** for ETL pipelines processing millions of documents
- **Eager validation** -- structure errors surface at construction time
- **21 ADF node types** with comprehensive formatting support

.. toctree::
   :maxdepth: 2
   :caption: Contents

   quickstart
   formats
   api
   changelog
