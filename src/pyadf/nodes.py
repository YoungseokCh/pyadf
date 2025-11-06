"""ADF node models with type-safe representations."""

import enum
import json
from pathlib import Path
from typing import Optional
from uuid import uuid4

from ._logger import get_logger

logger = get_logger()


class NodeType(enum.Enum):
    """Enumeration of ADF node types."""

    PARAGRAPH = (0, "paragraph")
    TEXT = (1, "text")
    HARD_BREAK = (2, "hardBreak")
    BULLET_LIST = (3, "bulletList")
    LIST_ITEM = (4, "listItem")
    PANEL = (5, "panel")
    TABLE = (6, "table")
    TABLE_ROW = (7, "tableRow")
    TABLE_HEADER = (8, "tableHeader")
    TABLE_CELL = (9, "tableCell")
    CODE_BLOCK = (10, "codeBlock")
    INLINE_CARD = (11, "inlineCard")
    TASK_LIST = (12, "taskList")
    TASK_ITEM = (13, "taskItem")
    ORDERED_LIST = (14, "orderedList")
    HEADING = (15, "heading")
    UNKNOWN = (16, "unknown")
    BLOCKQUOTE = (17, "blockquote")
    STATUS = (18, "status")

    def __str__(self) -> str:
        return self.value[1]

    @classmethod
    def from_string(cls, s: str) -> "NodeType":
        """Convert string to NodeType with O(1) cached lookup."""
        # Use a closure-based cache to avoid enum member conflicts
        if not hasattr(cls, "_cache"):
            cls._cache = {e.value[1]: e for e in cls}

        node_type = cls._cache.get(s)
        if node_type is None:
            raise ValueError(f"enum '{cls.__name__}' doesn't have value with string '{s}'")
        return node_type

    @classmethod
    def supported_values(cls) -> list[str]:
        """Get list of all supported node type strings."""
        return [e.value[1] for e in cls]


class UnsupportedNodeTypeError(Exception):
    """Raised when an unsupported node type is encountered."""

    pass


class InvalidNodeError(Exception):
    """Raised when node data is invalid or malformed."""

    pass


class Node:
    """Base class for all ADF nodes."""

    def __init__(self, node_dict: dict) -> None:
        if "type" not in node_dict:
            raise InvalidNodeError("node must contain 'type' attribute")

        self._type_str: str = node_dict["type"]

        try:
            n_type = NodeType.from_string(self._type_str)
        except ValueError:
            n_type = NodeType.UNKNOWN

        self._type: NodeType = n_type
        self._attrs: dict = node_dict.get("attrs", {})
        self._content: list = (
            node_dict["content"] if ("content" in node_dict) and (node_dict["content"]) else []
        )

        self._child_nodes: list["Node"] = []
        for child_node in self._content:
            child = create_node_from_dict(child_node)
            if child is not None:
                self._child_nodes.append(child)

    @property
    def type(self) -> NodeType:
        """Get the node type."""
        return self._type

    @property
    def child_nodes(self) -> list["Node"]:
        """Get child nodes."""
        return self._child_nodes


class ParagraphNode(Node):
    """Represents a paragraph node."""

    pass


class TextNode(Node):
    """Represents a text node with optional formatting marks."""

    def __init__(self, node_dict: dict) -> None:
        super().__init__(node_dict)

        if "text" not in node_dict:
            logger.warning("Text field does not exist in TextNode")
            self._text = ""
        else:
            self._text: str = node_dict["text"]

        self._marks: list[dict] = node_dict.get("marks", [])

        # Parse marks for common formatting
        self._is_bold = False
        self._is_italic = False
        self._is_link = False

        for mark in self._marks:
            mark_type = mark.get("type")
            if mark_type == "strong":
                self._is_bold = True
            elif mark_type == "em":
                self._is_italic = True
            elif mark_type == "link":
                self._is_link = True

    @property
    def text(self) -> str:
        """Get the text content."""
        return self._text

    @property
    def is_link(self) -> bool:
        """Check if text is a link."""
        return self._is_link

    @property
    def is_bold(self) -> bool:
        """Check if text is bold."""
        return self._is_bold

    @property
    def is_italic(self) -> bool:
        """Check if text is italic."""
        return self._is_italic


class HardBreakNode(Node):
    """Represents a hard line break."""

    pass


class ListNode(Node):
    """Base class for list nodes (bullet, ordered, task lists)."""

    def __init__(self, node_dict: dict) -> None:
        super().__init__(node_dict)

        self._elements: list[Node] = []
        for child_node in self._child_nodes:
            # Ensure we have only list items as children
            if child_node.type not in (NodeType.LIST_ITEM, NodeType.TASK_ITEM):
                logger.warning(
                    f"Expected LIST_ITEM or TASK_ITEM under list node, "
                    f"but got '{child_node.type}'"
                )
                continue

            self._elements.append(child_node)

    @property
    def elements(self) -> list[Node]:
        """Get list elements."""
        return self._elements


class BulletListNode(ListNode):
    """Represents a bullet list."""

    pass


class OrderedListNode(ListNode):
    """Represents an ordered (numbered) list."""

    pass


class TaskListNode(ListNode):
    """Represents a task list with checkboxes."""

    pass


class ListItemNode(Node):
    """Represents a list item."""

    pass


class TaskItemNode(Node):
    """Represents a task item with checkbox."""

    pass


class PanelNode(Node):
    """Represents a panel (info/warning/error box)."""

    pass


class BlockquoteNode(Node):
    """Represents a blockquote."""

    pass


class TableNode(Node):
    """Represents a table."""

    @property
    def header(self) -> Optional["TableRowNode"]:
        """Get the table header row if it exists."""

        def has_header_cells(node: Optional[Node]) -> bool:
            if node is None or node.type != NodeType.TABLE_ROW:
                return False
            return any(child.type == NodeType.TABLE_HEADER for child in node.child_nodes)

        headers = [node for node in self.child_nodes if has_header_cells(node)]

        if len(headers) == 0:
            return None

        if len(headers) > 1:
            logger.warning("table contains more than one header")

        return headers[0] if isinstance(headers[0], TableRowNode) else None


class TableRowNode(Node):
    """Represents a table row."""

    @property
    def column_count(self) -> int:
        """Get the number of columns in this row."""
        count = 0
        for child in self.child_nodes:
            if child.type in (NodeType.TABLE_HEADER, NodeType.TABLE_CELL):
                if isinstance(child, (TableHeaderNode, TableCellNode)):
                    count += child.colspan
                else:
                    count += 1
        return count


class TableCellNode(Node):
    """Represents a table cell."""

    @property
    def colspan(self) -> int:
        """Get the column span of this cell."""
        result = self._attrs.get("colspan", 1)
        return int(result) if result is not None else 1


class TableHeaderNode(TableCellNode):
    """Represents a table header cell."""

    pass


class CodeBlockNode(Node):
    """Represents a code block."""

    @property
    def language(self) -> Optional[str]:
        """Get the programming language of the code block."""
        return self._attrs.get("language")


class InlineCardNode(Node):
    """Represents an inline card (link preview)."""

    @property
    def url(self) -> Optional[str]:
        """Get the URL of the inline card."""
        return self._attrs.get("url")

    @property
    def data(self) -> Optional[str]:
        """Get the data of the inline card."""
        return self._attrs.get("data")


class HeadingNode(Node):
    """Represents a heading."""

    @property
    def level(self) -> int:
        """Get the heading level (1-6)."""
        return self._attrs.get("level", 1)


class StatusNode(Node):
    """Represents a status badge."""

    @property
    def status_text(self) -> str:
        """Get the status text."""
        return self._attrs.get("text", "")

    @property
    def color(self) -> str:
        """Get the status color."""
        return self._attrs.get("color", "")


# Node registry for factory pattern
_NODE_REGISTRY: dict[NodeType, type[Node]] = {
    NodeType.PARAGRAPH: ParagraphNode,
    NodeType.TEXT: TextNode,
    NodeType.HARD_BREAK: HardBreakNode,
    NodeType.BULLET_LIST: BulletListNode,
    NodeType.ORDERED_LIST: OrderedListNode,
    NodeType.TASK_LIST: TaskListNode,
    NodeType.LIST_ITEM: ListItemNode,
    NodeType.TASK_ITEM: TaskItemNode,
    NodeType.PANEL: PanelNode,
    NodeType.BLOCKQUOTE: BlockquoteNode,
    NodeType.TABLE: TableNode,
    NodeType.TABLE_ROW: TableRowNode,
    NodeType.TABLE_HEADER: TableHeaderNode,
    NodeType.TABLE_CELL: TableCellNode,
    NodeType.CODE_BLOCK: CodeBlockNode,
    NodeType.INLINE_CARD: InlineCardNode,
    NodeType.HEADING: HeadingNode,
    NodeType.STATUS: StatusNode,
}


# Known unsupported node types (silently handled)
_KNOWN_UNSUPPORTED_TYPES = {
    "mediaSingle",
    "mediaGroup",
    "mediaInline",
    "expand",
    "rule",
    "media",
    "mention",
    "emoji",
    "embedCard",
}


def create_node_from_dict(node_dict: dict) -> Optional[Node]:
    """
    Create a node from a dictionary using registry pattern.

    Args:
        node_dict: Dictionary containing node data

    Returns:
        Node instance or None if node cannot be created

    Raises:
        UnsupportedNodeTypeError: If node type is not supported
        InvalidNodeError: If node data is invalid
    """
    if "type" not in node_dict:
        return None

    node_type_str = node_dict["type"]

    # Handle known unsupported types gracefully
    if node_type_str in _KNOWN_UNSUPPORTED_TYPES:
        logger.debug(f"Skipping known unsupported node type: {node_type_str}")
        return Node(node_dict)

    try:
        node_type = NodeType.from_string(node_type_str)
    except ValueError as e:
        # Log debug info if needed
        logger.error(f"Unknown node type: {node_type_str}")
        _write_debug_file(node_dict, node_type_str)
        raise UnsupportedNodeTypeError(f"Unsupported node type: {node_type_str}") from e

    # Get the appropriate node class from registry
    node_class = _NODE_REGISTRY.get(node_type)

    if node_class is None:
        logger.error(f"No handler registered for node type: {node_type}")
        raise UnsupportedNodeTypeError(f"No handler for node type: {node_type}")

    try:
        return node_class(node_dict)
    except Exception as e:
        logger.error(f"Error creating node of type {node_type}: {e}")
        _write_debug_file(node_dict, node_type_str)
        raise


def create_nodes_from_list(node_dict_list: list[dict]) -> list[Node]:
    """
    Create a list of nodes from a list of dictionaries.

    Args:
        node_dict_list: List of dictionaries containing node data

    Returns:
        List of created nodes (None values filtered out)
    """
    nodes = []
    for node_dict in node_dict_list:
        try:
            node = create_node_from_dict(node_dict)
            if node is not None:
                nodes.append(node)
        except UnsupportedNodeTypeError:
            # Skip unsupported nodes
            continue

    return nodes


def _write_debug_file(node_dict: dict, node_type: str) -> None:
    """Write debug file for problematic nodes."""
    try:
        debug_dir = Path("/tmp")
        if debug_dir.exists():
            filename = debug_dir / f"pyadf_debug_{node_type}_{uuid4()}.json"
            with open(filename, "w") as f:
                json.dump(node_dict, f, indent=4)
            logger.debug(f"Debug file written: {filename}")
    except Exception as e:
        logger.debug(f"Could not write debug file: {e}")
