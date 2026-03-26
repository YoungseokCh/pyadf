Quick Start
===========

Installation
------------

.. code-block:: bash

   pip install pyadf

Prebuilt wheels are available for Linux and macOS (x86_64 and aarch64) and Windows (x86_64).
Requires Python 3.11+.

Basic Usage
-----------

Convert ADF to Markdown
~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: python

   from pyadf import Document

   adf_data = {
       "type": "doc",
       "content": [
           {
               "type": "paragraph",
               "content": [
                   {"type": "text", "text": "Hello, "},
                   {"type": "text", "text": "world!", "marks": [{"type": "strong"}]},
               ],
           }
       ],
   }

   doc = Document(adf_data)
   print(doc.to_markdown())
   # Output: Hello, **world!**

You can also pass a JSON string:

.. code-block:: python

   doc = Document('{"type": "doc", "content": [...]}')

Convert Jira Markup to Markdown
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: python

   from pyadf import Document

   doc = Document("h1. Hello *world*", format="jira")
   print(doc.to_markdown())
   # Output: # Hello **world**

Convert Markdown
~~~~~~~~~~~~~~~~

.. code-block:: python

   from pyadf import Document

   doc = Document("# Hello **world**", format="markdown")
   print(doc.to_markdown())  # # Hello **world**

Convert HTML/XHTML
~~~~~~~~~~~~~~~~~~

Works with Confluence storage format and standard HTML:

.. code-block:: python

   from pyadf import Document

   doc = Document("<h1>Hello <b>world</b></h1>", format="html")
   print(doc.to_markdown())  # # Hello **world**

Convert Markdown to Jira Markup
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

To convert Markdown back to Jira markup:

.. code-block:: python

   from pyadf import markdown_to_jira

   jira = markdown_to_jira("## Section\n**bold text**")
   # "h2. Section\n*bold text*"

Configuring Output
~~~~~~~~~~~~~~~~~~

.. code-block:: python

   from pyadf import Document, MarkdownConfig

   doc = Document(adf_data)

   # Use * for bullet lists (default is +)
   config = MarkdownConfig(bullet_marker="*")
   doc.to_markdown(config)  # "* Item 1\n* Item 2"

   # Show full link URLs
   config = MarkdownConfig(show_links=True)
   doc.to_markdown(config)  # "[Link text](http://example.com)"

.. list-table:: Configuration Options
   :header-rows: 1

   * - Option
     - Values
     - Default
     - Description
   * - ``bullet_marker``
     - ``+``, ``-``, ``*``
     - ``+``
     - Character for bullet list items
   * - ``show_links``
     - ``True``, ``False``
     - ``False``
     - Include link URLs in output

Batch Processing
~~~~~~~~~~~~~~~~

For ETL pipelines processing large volumes of ADF documents:

.. code-block:: python

   from pyadf import convert_jsonl, ConversionError

   # From a JSONL file (one ADF document per line)
   for result in convert_jsonl("export.jsonl"):
       if isinstance(result, ConversionError):
           print(f"Line {result.line_number}: {result.error}")
       else:
           print(result)

Error Handling
~~~~~~~~~~~~~~

All parsing errors are raised at construction time:

.. code-block:: python

   from pyadf import Document, InvalidJSONError, UnsupportedNodeTypeError

   try:
       doc = Document("invalid json")
   except InvalidJSONError as e:
       print(f"Bad JSON: {e}")

   try:
       doc = Document({"type": "unsupported_type"})
   except UnsupportedNodeTypeError as e:
       print(f"Unknown node: {e}")
