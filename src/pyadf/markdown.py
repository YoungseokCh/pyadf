"""Markdown generation from ADF nodes using presenter pattern."""

from dataclasses import dataclass
from typing import Optional

from .nodes import (
    CodeBlockNode,
    HeadingNode,
    InlineCardNode,
    Node,
    NodeType,
    StatusNode,
    TableRowNode,
    TextNode,
)


@dataclass
class RenderContext:
    """Context for rendering markdown, replacing multiple boolean flags."""

    is_first: bool = False
    is_prev_hard_break: bool = False
    parent_node: Optional[Node] = None


def gen_md_from_root_node(root_node: Node) -> str:
    """
    Generate markdown from a root ADF node.

    Args:
        root_node: The root node to convert

    Returns:
        Markdown string representation
    """
    try:
        root_node_presenter = create_node_presenter_from_node(
            root_node, RenderContext(is_first=True)
        )
    except Exception:
        return ""

    return str(root_node_presenter)


class NodePresenter:
    """Base presenter for converting ADF nodes to markdown."""

    def __init__(self, node: Node, context: Optional[RenderContext] = None) -> None:
        self._node = node
        self._child_presenters: list[NodePresenter] = []

        cur_node_type = None
        for idx, child in enumerate(self._node.child_nodes):
            child_context = RenderContext(
                is_first=(idx == 0),
                is_prev_hard_break=(cur_node_type == NodeType.HARD_BREAK),
                parent_node=self._node,
            )
            child_presenter = create_node_presenter_from_node(child, child_context)
            if child_presenter:
                self._child_presenters.append(child_presenter)

            cur_node_type = child.type

    def __str__(self) -> str:
        return "".join(str(cp) for cp in self._child_presenters)

    @property
    def node(self) -> Node:
        """Get the underlying node."""
        return self._node

    @property
    def child_presenters(self) -> list["NodePresenter"]:
        """Get child presenters."""
        return self._child_presenters


class ParagraphPresenter(NodePresenter):
    """Presenter for paragraph nodes."""

    def __init__(self, node: Node, context: Optional[RenderContext] = None) -> None:
        super().__init__(node, context)
        self._no_leading_newlines = self._should_skip_leading_newline(context)

    def _should_skip_leading_newline(self, context: Optional[RenderContext]) -> bool:
        """Determine if leading newline should be skipped."""
        if context is None:
            return False

        return (
            context.parent_node is None
            or context.is_first
            or context.is_prev_hard_break
            or (context.parent_node and context.parent_node.type == NodeType.LIST_ITEM)
        )

    def __str__(self) -> str:
        out = "" if self._no_leading_newlines else "\n"
        out += "".join(str(cp) for cp in self._child_presenters)
        return out


class TextPresenter(NodePresenter):
    """Presenter for text nodes."""

    def __init__(self, node: Node, context: Optional[RenderContext] = None) -> None:
        if not isinstance(node, TextNode):
            raise ValueError("node is not a TextNode")
        super().__init__(node, context)
        self._text_node = node

    def __str__(self) -> str:
        out = self._text_node.text

        if self._text_node.is_bold:
            out = _apply_formatting(out, "**")

        if self._text_node.is_italic:
            out = _apply_formatting(out, "*")

        if self._text_node.is_link:
            out = f"[{out}]"

        return out


class HardBreakPresenter(NodePresenter):
    """Presenter for hard break nodes."""

    def __str__(self) -> str:
        return "  \n"


class BulletListPresenter(NodePresenter):
    """Presenter for bullet list nodes."""

    def __str__(self) -> str:
        bulleted_list = [f"+ {str(cp)}" for cp in self._child_presenters]
        return "\n".join(bulleted_list)


class OrderedListPresenter(NodePresenter):
    """Presenter for ordered (numbered) list nodes."""

    def __str__(self) -> str:
        ordered_list = [f"{idx + 1}. {str(cp)}" for idx, cp in enumerate(self._child_presenters)]
        return "\n".join(ordered_list)


class TaskListPresenter(NodePresenter):
    """Presenter for task list nodes."""

    def __str__(self) -> str:
        # Task lists use checkbox syntax
        task_list = [f"- [ ] {str(cp)}" for cp in self._child_presenters]
        return "\n".join(task_list)


class ListItemPresenter(NodePresenter):
    """Presenter for list item nodes."""

    pass


class PanelPresenter(NodePresenter):
    """Presenter for panel nodes."""

    def __str__(self) -> str:
        out_lines = []
        for child_presenter in self._child_presenters:
            cur_presenter_lines = str(child_presenter).splitlines()
            for line in cur_presenter_lines:
                out_lines.append(f"> {line}")

        return "\n".join(out_lines)


class BlockquotePresenter(NodePresenter):
    """Presenter for blockquote nodes."""

    def __str__(self) -> str:
        out_lines = []
        for child_presenter in self._child_presenters:
            cur_presenter_str = str(child_presenter)
            # Strip leading/trailing whitespace and split into lines
            cur_presenter_lines = cur_presenter_str.strip().splitlines()
            for line in cur_presenter_lines:
                # Add > prefix to each line, even if empty
                out_lines.append(f"> {line}")

        return "\n".join(out_lines)


class TablePresenter(NodePresenter):
    """Presenter for table nodes."""

    def __str__(self) -> str:
        row_list = []
        for row_presenter in self._child_presenters:
            row_list.append(str(row_presenter))

            # Check if this row is a header row
            is_header = any(
                child.node.type == NodeType.TABLE_HEADER
                for child in row_presenter.child_presenters
            )

            if is_header:
                # Insert separator: | --- | --- | --- |
                col_count = row_presenter.column_count
                row_list.append(f"| {' | '.join(['---'] * col_count)} |")

        return "\n".join(row_list)


class TableRowPresenter(NodePresenter):
    """Presenter for table row nodes."""

    def __init__(self, node: Node, context: Optional[RenderContext] = None) -> None:
        if not isinstance(node, TableRowNode):
            raise ValueError("node is not a TableRowNode")
        super().__init__(node, context)
        self._row_node = node

    def __str__(self) -> str:
        cells = [str(cp) for cp in self._child_presenters]
        return f"| {' | '.join(cells)} |"

    @property
    def column_count(self) -> int:
        """Get the number of columns in this row."""
        return self._row_node.column_count


class TableCellPresenter(NodePresenter):
    """Presenter for table cell nodes (both header and regular cells)."""

    pass


class CodeBlockPresenter(NodePresenter):
    """Presenter for code block nodes."""

    def __init__(self, node: Node, context: Optional[RenderContext] = None) -> None:
        if not isinstance(node, CodeBlockNode):
            raise ValueError("node is not a CodeBlockNode")
        super().__init__(node, context)
        self._code_node = node

    def __str__(self) -> str:
        content = "".join(str(cp) for cp in self._child_presenters)
        lang = self._code_node.language

        if lang:
            return f"```{lang}\n{content}\n```"
        else:
            return f"```\n{content}\n```"


class InlineCardPresenter(NodePresenter):
    """Presenter for inline card nodes."""

    def __init__(self, node: Node, context: Optional[RenderContext] = None) -> None:
        if not isinstance(node, InlineCardNode):
            raise ValueError("node is not an InlineCardNode")
        super().__init__(node, context)
        self._inline_card = node

    def __str__(self) -> str:
        if self._inline_card.url:
            return f"[{self._inline_card.url}]"
        elif self._inline_card.data:
            return f"```\n{self._inline_card.data}\n```"
        else:
            return "<broken_inlinecard>"


class HeadingPresenter(NodePresenter):
    """Presenter for heading nodes."""

    def __init__(self, node: Node, context: Optional[RenderContext] = None) -> None:
        if not isinstance(node, HeadingNode):
            raise ValueError("node is not a HeadingNode")
        super().__init__(node, context)
        self._heading = node

    def __str__(self) -> str:
        content = "".join(str(cp) for cp in self._child_presenters)
        level = max(1, min(6, self._heading.level))  # Clamp to 1-6
        prefix = "#" * level
        return f"{prefix} {content}"


class StatusPresenter(NodePresenter):
    """Presenter for status badge nodes."""

    def __init__(self, node: Node, context: Optional[RenderContext] = None) -> None:
        if not isinstance(node, StatusNode):
            raise ValueError("node is not a StatusNode")
        super().__init__(node, context)
        self._status_node = node

    def __str__(self) -> str:
        status_text = self._status_node.status_text
        return f"**[{status_text}]**"


# Presenter registry for factory pattern
_PRESENTER_REGISTRY: dict[NodeType, type[NodePresenter]] = {
    NodeType.PARAGRAPH: ParagraphPresenter,
    NodeType.TEXT: TextPresenter,
    NodeType.HARD_BREAK: HardBreakPresenter,
    NodeType.BULLET_LIST: BulletListPresenter,
    NodeType.ORDERED_LIST: OrderedListPresenter,
    NodeType.TASK_LIST: TaskListPresenter,
    NodeType.LIST_ITEM: ListItemPresenter,
    NodeType.TASK_ITEM: ListItemPresenter,  # Same as regular list items
    NodeType.PANEL: PanelPresenter,
    NodeType.BLOCKQUOTE: BlockquotePresenter,
    NodeType.TABLE: TablePresenter,
    NodeType.TABLE_ROW: TableRowPresenter,
    NodeType.TABLE_HEADER: TableCellPresenter,
    NodeType.TABLE_CELL: TableCellPresenter,
    NodeType.CODE_BLOCK: CodeBlockPresenter,
    NodeType.INLINE_CARD: InlineCardPresenter,
    NodeType.HEADING: HeadingPresenter,
    NodeType.STATUS: StatusPresenter,
}


def create_node_presenter_from_node(
    node: Node, context: Optional[RenderContext] = None
) -> Optional[NodePresenter]:
    """
    Create a presenter for a node using registry pattern.

    Args:
        node: The node to create a presenter for
        context: Rendering context

    Returns:
        NodePresenter instance or None if node type is unsupported
    """
    presenter_class = _PRESENTER_REGISTRY.get(node.type)

    if presenter_class is None:
        # Fallback to base presenter for unsupported types
        return NodePresenter(node, context)

    return presenter_class(node, context)


def _apply_formatting(text: str, format_symbols: str) -> str:
    """
    Apply markdown formatting symbols to text.

    Handles trailing spaces properly by moving them outside the formatting.

    Args:
        text: Text to format
        format_symbols: Formatting symbols (e.g., "**" for bold, "*" for italic)

    Returns:
        Formatted text with trailing spaces preserved outside formatting
    """
    text, trailing_spaces_count = _remove_trailing_spaces(text)
    return f"{format_symbols}{text}{format_symbols}{' ' * trailing_spaces_count}"


def _remove_trailing_spaces(text: str) -> tuple[str, int]:
    """
    Remove trailing spaces from text and return count.

    Args:
        text: Text to process

    Returns:
        Tuple of (text without trailing spaces, count of trailing spaces)
    """
    count = 0
    for ch in reversed(text):
        if ch == " ":
            count += 1
        else:
            break

    # Remove trailing spaces and return that string;
    # str[:0] will clear the string, take it into account by using if ... else
    return (text[:-count] if count > 0 else text, count)
