Supported Formats
=================

ADF (Atlassian Document Format)
-------------------------------

ADF is the structured JSON format used by Jira and Confluence for rich content.
pyadf supports 21 node types:

.. list-table::
   :header-rows: 1

   * - ADF Node Type
     - Markdown Output
     - Notes
   * - ``doc``
     - Document root
     - Top-level container
   * - ``paragraph``
     - Plain text with newlines
     -
   * - ``text``
     - Formatted text
     - Supports marks (see below)
   * - ``heading``
     - ``# Heading`` (levels 1--6)
     -
   * - ``bulletList``
     - ``+ Item``
     -
   * - ``orderedList``
     - ``1. Item``
     -
   * - ``taskList``
     - ``- [ ] Task``
     - Checkbox tasks
   * - ``codeBlock``
     - Fenced code block
     - Optional language syntax
   * - ``blockquote``
     - ``> Quote``
     -
   * - ``panel``
     - ``> Panel content``
     - Info/warning/error boxes
   * - ``table``
     - Markdown table
     - Supports headers and colspan
   * - ``status``
     - ``**[STATUS]**``
     - Status badges
   * - ``inlineCard``
     - ``[link]`` or code block
     - Link previews
   * - ``emoji``
     - Unicode emoji
     -
   * - ``hardBreak``
     - Line break
     -
   * - ``mention``
     - ``@DisplayName``
     - Jira user mentions

Text Marks
~~~~~~~~~~

Text nodes support formatting via marks:

.. list-table::
   :header-rows: 1

   * - Mark Type
     - Markdown Output
   * - ``strong``
     - ``**bold**``
   * - ``em``
     - ``*italic*``
   * - ``link``
     - ``[text](url)`` (when ``show_links=True``)
   * - ``code``
     - `` `code` ``
   * - ``strike``
     - ``~~strikethrough~~``
   * - ``underline``
     - ``<ins>text</ins>``
   * - ``superscript``
     - ``<sup>text</sup>``
   * - ``subsup``
     - ``<sub>text</sub>``
   * - ``textColor``
     - ``<span style="color:X">text</span>``

Jira Wiki Markup
----------------

Jira wiki markup is the older wiki-style syntax used in Jira descriptions and comments.
Use ``format="jira"`` with ``Document`` or the standalone conversion functions.

Supported Conversions (Jira → Markdown)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1

   * - Jira Markup
     - Markdown Output
   * - ``h1. Title``
     - ``# Title``
   * - ``*bold*``
     - ``**bold**``
   * - ``_italic_``
     - ``*italic*``
   * - ``{{code}}``
     - `` `code` ``
   * - ``{code:python}...{code}``
     - Fenced code block with language
   * - ``{noformat}...{noformat}``
     - Fenced code block
   * - ``bq. text``
     - ``> text``
   * - ``{quote}...{quote}``
     - Block quote (each line prefixed ``>``)
   * - ``[text|url]``
     - ``[text](url)``
   * - ``!image.png!``
     - ``![](image.png)``
   * - ``* item``
     - ``- item``
   * - ``# item``
     - ``1. item``
   * - ``||header||``
     - Markdown table header + separator
   * - ``-text-``
     - ``-text-`` (passthrough)
   * - ``+text+``
     - ``<ins>text</ins>``
   * - ``^text^``
     - ``<sup>text</sup>``
   * - ``~text~``
     - ``<sub>text</sub>``
   * - ``??text??``
     - ``<cite>text</cite>``
   * - ``{color:red}text{color}``
     - ``<span style="color:red">text</span>``

Supported Conversions (Markdown → Jira)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The reverse conversion is available via :func:`pyadf.markdown_to_jira`:

.. code-block:: python

   from pyadf import markdown_to_jira

   jira = markdown_to_jira("## Section\n**bold text**")
   # "h2. Section\n*bold text*"
