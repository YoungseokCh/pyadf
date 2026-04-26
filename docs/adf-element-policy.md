# ADF Element Policy

This document tracks pyadf policy for Jira ADF nodes and marks, including both
ADF rendering and Markdown import behavior.

## Markdown Import Principles

- Canonical Markdown rendering is the contract. Accepted non-canonical input may
  normalize to pyadf's renderer output.
- Exact Markdown roundtrip is guaranteed only for canonical forms.
- Generic HTML is rejected.
- pyadf fallback HTML is narrow and intentional. Only pyadf-controlled fallback
  wrappers are accepted: `<div adf="..." ...></div>` at block/root level and
  `<span adf="..." ...></span>` in inline or table-cell contexts.
- Fallback wrappers must be well-formed, and `params` must be a valid JSON
  object when present.
- Markdown import does not infer ambiguous ADF nodes from visually similar
  Markdown. For example, blockquote-like Markdown maps to `blockquote`, not
  `panel`.

Status meanings:

- `native`: parsed from ADF and rendered to Markdown without fallback
- `markdown-import`: accepted by `Document.from_markdown(...)`
- `html-fallback`: recognized as a known unsupported ADF node and preservable with `on_known_unsupported="html"`
- `defer`: valid Jira ADF element, but pyadf policy or Markdown mapping is not complete
- `reject`: intentionally not accepted from Markdown import

## Nodes

| Jira ADF element | ADF render policy | Markdown import policy | Notes |
| --- | --- | --- | --- |
| `doc`[^node-doc] | native | markdown-import | Document root. |
| `paragraph`[^node-paragraph] | native | markdown-import | Basic block container. |
| `text`[^node-text] | native | markdown-import | Supports documented mark subset below. |
| `hardBreak`[^node-hardbreak] | native | markdown-import | Markdown hard break: two trailing spaces plus newline. |
| `heading`[^node-heading] | native | markdown-import | ATX headings only. |
| `blockquote`[^node-blockquote] | native | markdown-import | Markdown blockquote maps back to `blockquote`, not `panel`. |
| `bodiedSyncBlock` | defer | defer | Listed in the structure index, but not yet represented in pyadf's native node enum. |
| `bulletList`[^node-bulletlist] | native | markdown-import | Includes representative nested-list support. |
| `orderedList`[^node-orderedlist] | native | markdown-import | Source numbering is normalized by renderer. |
| `listItem`[^node-listitem] | native | markdown-import | Includes representative nested and multi-paragraph list-item support. |
| `codeBlock`[^node-codeblock] | native | markdown-import | Fenced code blocks; info string canonicalizes to first language token. |
| `syncBlock` | defer | defer | Listed in the structure index, but not yet represented in pyadf's native node enum. |
| `multiBodiedExtension` | defer | defer | Listed in the structure index, but not yet represented in pyadf's native node enum. |
| `table`[^node-table] | native | markdown-import | Canonical GFM tables with inline marks are supported. |
| `tableRow`[^node-tablerow] | native | markdown-import | See `table`. |
| `tableCell`[^node-tablecell] | native | markdown-import | See `table`. |
| `tableHeader`[^node-tableheader] | native | markdown-import | Header rows are parsed from the GFM table head. |
| `panel`[^node-panel] | native | reject | Renders like blockquote, but Markdown import must not infer panel. |
| `blockTaskItem` | defer | defer | Listed in the structure index, but not yet represented in pyadf's native node enum. |
| `taskList` | native | markdown-import | pyadf-supported node not listed in the current structure index; `localId` is preserved from ADF when present and not generated from Markdown. |
| `taskItem` | native | markdown-import | pyadf-supported node not listed in the current structure index; `state` maps to `- [ ]` / `- [x]`, and `localId` is preserved from ADF when present. |
| `inlineCard`[^node-inlinecard] | native | reject | Markdown-like output is ambiguous with plain links/text. |
| `blockCard` | native | reject | Markdown-like output is ambiguous with plain links/text. |
| `status`[^node-status] | native | reject | Markdown-like output is ambiguous with plain formatted text. |
| `emoji`[^node-emoji] | native | reject | Markdown import treats emoji text as plain text. |
| `mention`[^node-mention] | native | reject | Markdown import treats mention text as plain text. |
| `date`[^node-date] | defer | defer | Jira element not yet represented in pyadf's native node enum. |
| `nestedExpand`[^node-nestedexpand] | defer | defer | Jira element not yet represented in pyadf's native node enum. |
| `extensionFrame` | defer | defer | Listed in the structure index, but not yet represented in pyadf's native node enum. |
| `expand`[^node-expand] | html-fallback | reject | Known unsupported node; fallback HTML only. |
| `rule`[^node-rule] | html-fallback | reject | Markdown thematic breaks are rejected. |
| `media`[^node-media] | html-fallback | reject | Known unsupported node; fallback HTML only. |
| `mediaSingle`[^node-mediasingle] | html-fallback | reject | Known unsupported node; fallback HTML only. |
| `mediaGroup`[^node-mediagroup] | html-fallback | reject | Known unsupported node; fallback HTML only. |
| `mediaInline`[^node-mediainline] | html-fallback | reject | Known unsupported node; fallback HTML only. |
| `embedCard` | html-fallback | reject | Known unsupported node; fallback HTML only. |
| `extension` | html-fallback | markdown-import | Only pyadf fallback wrappers are accepted, not generic HTML. |

## Marks

| Jira ADF element | ADF render policy | Markdown import policy | Notes |
| --- | --- | --- | --- |
| `strong`[^mark-strong] | native | markdown-import | `**x**`; underscore input canonicalizes to asterisk output. |
| `em`[^mark-em] | native | markdown-import | `*x*`; underscore input canonicalizes to asterisk output. |
| `link`[^mark-link] | native | markdown-import | Inline links and URL autolinks are accepted; reference links are rejected. |
| `code`[^mark-code] | native | markdown-import | Inline code. Per Jira docs, can only combine with `link`. |
| `strike`[^mark-strike] | native | markdown-import | `~~x~~`. |
| `border` | defer | reject | Listed in the structure index; attrs-aware mark model support is needed. |
| `underline`[^mark-underline] | defer | reject | Jira mark exists; no canonical Markdown import contract. |
| `textColor`[^mark-textcolor] | defer | reject | Attrs-aware mark model support is needed. |
| `backgroundColor`[^mark-backgroundcolor] | defer | reject | Present in the side navigation; attrs-aware mark model support is needed. |
| `subsup`[^mark-subsup] | defer | reject | Attrs-aware mark model support is needed. |

## Gap Notes

- The structure index lists `blockTaskItem`, while pyadf currently supports `taskList` / `taskItem`. Treat this as an ADF-variant gap to revisit against product behavior and schema examples.
- pyadf's current mark model preserves mark type and link href. Marks that require richer attrs, such as `border`, `textColor`, `backgroundColor`, and `subsup`, need a mark-attrs model before they can be native-rendered or imported safely.
- Some structure-index entries currently have no stable detail page under the documented URL pattern, so they are listed without footnotes until an official detail page is available.

## References

[^node-blockquote]: Jira blockquote node: <https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/blockquote/>
[^node-bulletlist]: Jira bulletList node: <https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/bulletList/>
[^node-codeblock]: Jira codeBlock node: <https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/codeBlock/>
[^node-date]: Jira date node: <https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/date/>
[^node-doc]: Jira doc node: <https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/doc/>
[^node-emoji]: Jira emoji node: <https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/emoji/>
[^node-expand]: Jira expand node: <https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/expand/>
[^node-hardbreak]: Jira hardBreak node: <https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/hardBreak/>
[^node-heading]: Jira heading node: <https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/heading/>
[^node-inlinecard]: Jira inlineCard node: <https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/inlineCard/>
[^node-listitem]: Jira listItem node: <https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/listItem/>
[^node-media]: Jira media node: <https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/media/>
[^node-mediagroup]: Jira mediaGroup node: <https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/mediaGroup/>
[^node-mediasingle]: Jira mediaSingle node: <https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/mediaSingle/>
[^node-mediainline]: Jira mediaInline node: <https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/mediaInline/>
[^node-mention]: Jira mention node: <https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/mention/>
[^node-nestedexpand]: Jira nestedExpand node: <https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/nestedExpand/>
[^node-orderedlist]: Jira orderedList node: <https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/orderedList/>
[^node-panel]: Jira panel node: <https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/panel/>
[^node-paragraph]: Jira paragraph node: <https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/paragraph/>
[^node-rule]: Jira rule node: <https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/rule/>
[^node-status]: Jira status node: <https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/status/>
[^node-table]: Jira table node: <https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/table/>
[^node-tablecell]: Jira tableCell node: <https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/table_cell/>
[^node-tableheader]: Jira tableHeader node: <https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/table_header/>
[^node-tablerow]: Jira tableRow node: <https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/table_row/>
[^node-text]: Jira text node: <https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/text/>
[^mark-backgroundcolor]: Jira backgroundColor mark: <https://developer.atlassian.com/cloud/jira/platform/apis/document/marks/backgroundColor/>
[^mark-code]: Jira code mark: <https://developer.atlassian.com/cloud/jira/platform/apis/document/marks/code/>
[^mark-em]: Jira em mark: <https://developer.atlassian.com/cloud/jira/platform/apis/document/marks/em/>
[^mark-link]: Jira link mark: <https://developer.atlassian.com/cloud/jira/platform/apis/document/marks/link/>
[^mark-strike]: Jira strike mark: <https://developer.atlassian.com/cloud/jira/platform/apis/document/marks/strike/>
[^mark-strong]: Jira strong mark: <https://developer.atlassian.com/cloud/jira/platform/apis/document/marks/strong/>
[^mark-subsup]: Jira subsup mark: <https://developer.atlassian.com/cloud/jira/platform/apis/document/marks/subsup/>
[^mark-textcolor]: Jira textColor mark: <https://developer.atlassian.com/cloud/jira/platform/apis/document/marks/textColor/>
[^mark-underline]: Jira underline mark: <https://developer.atlassian.com/cloud/jira/platform/apis/document/marks/underline/>
