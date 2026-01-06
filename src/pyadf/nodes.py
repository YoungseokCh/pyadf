"""ADF node models with type-safe representations."""

import enum
from typing import Optional

from ._logger import get_logger
from .exceptions import (
    InvalidFieldError,
    MissingFieldError,
    NodeCreationError,
    UnsupportedNodeTypeError,
)

logger = get_logger()


class NodeType(enum.Enum):
    """Enumeration of ADF node types."""

    DOC = (0, "doc")
    PARAGRAPH = (1, "paragraph")
    TEXT = (2, "text")
    HARD_BREAK = (3, "hardBreak")
    BULLET_LIST = (4, "bulletList")
    LIST_ITEM = (5, "listItem")
    PANEL = (6, "panel")
    TABLE = (7, "table")
    TABLE_ROW = (8, "tableRow")
    TABLE_HEADER = (9, "tableHeader")
    TABLE_CELL = (10, "tableCell")
    CODE_BLOCK = (11, "codeBlock")
    INLINE_CARD = (12, "inlineCard")
    TASK_LIST = (13, "taskList")
    TASK_ITEM = (14, "taskItem")
    ORDERED_LIST = (15, "orderedList")
    HEADING = (16, "heading")
    UNKNOWN = (17, "unknown")
    BLOCKQUOTE = (18, "blockquote")
    STATUS = (19, "status")
    EMOJI = (20, "emoji")

    def __str__(self) -> str:
        return self.value[1]

    @classmethod
    def from_string(cls, s: str) -> "NodeType":
        """Convert string to NodeType with O(1) cached lookup.

        Args:
            s: String representation of the node type

        Returns:
            NodeType enum member

        Raises:
            InvalidFieldError: If the string doesn't match any known node type
        """
        # Use a closure-based cache to avoid enum member conflicts
        if not hasattr(cls, "_cache"):
            cls._cache = {e.value[1]: e for e in cls}

        node_type = cls._cache.get(s)
        if node_type is None:
            raise InvalidFieldError(
                field_name="type",
                invalid_value=s,
                expected_values=cls.supported_values(),
            )
        return node_type

    @classmethod
    def supported_values(cls) -> list[str]:
        """Get list of all supported node type strings."""
        return [e.value[1] for e in cls]


class Node:
    """Base class for all ADF nodes."""

    def __init__(self, node_dict: dict, node_path: str = "") -> None:
        """
        Initialize a node from dictionary.

        Args:
            node_dict: Dictionary containing node data
            node_path: Path to this node in the ADF tree (for error reporting)

        Raises:
            MissingFieldError: If required 'type' field is missing
            InvalidFieldError: If 'type' field has an invalid value
        """
        if "type" not in node_dict:
            raise MissingFieldError(
                field_name="type",
                node_path=node_path or "<root>",
                expected_values=NodeType.supported_values(),
            )

        self._type_str: str = node_dict["type"]
        self._node_path: str = node_path

        try:
            n_type = NodeType.from_string(self._type_str)
        except InvalidFieldError as e:
            # Re-raise with node path context
            e.node_path = node_path or "<root>"
            n_type = NodeType.UNKNOWN
            # For unknown types in base Node, we'll log but continue
            logger.debug(f"Unknown node type '{self._type_str}' at {node_path}")

        self._type: NodeType = n_type
        self._attrs: dict = node_dict.get("attrs", {})
        self._content: list = (
            node_dict["content"] if ("content" in node_dict) and (node_dict["content"]) else []
        )

        self._child_nodes: list[Node] = []
        for idx, child_node in enumerate(self._content):
            child_path = f"{node_path} > {self._type_str}[{idx}]" if node_path else self._type_str
            try:
                child = create_node_from_dict(child_node, child_path)
                if child is not None:
                    self._child_nodes.append(child)
            except (MissingFieldError, InvalidFieldError, UnsupportedNodeTypeError):
                # Re-raise validation errors with proper context
                raise
            except Exception as e:
                # Wrap unexpected errors
                logger.error(f"Error creating child node at {child_path}: {e}")
                raise

    @property
    def type(self) -> NodeType:
        """Get the node type."""
        return self._type

    @property
    def child_nodes(self) -> list["Node"]:
        """Get child nodes."""
        return self._child_nodes


class DocNode(Node):
    """Represents a document root node."""

    pass


class ParagraphNode(Node):
    """Represents a paragraph node."""

    pass


class TextNode(Node):
    """Represents a text node with optional formatting marks."""

    def __init__(self, node_dict: dict, node_path: str = "") -> None:
        super().__init__(node_dict, node_path)

        if "text" not in node_dict:
            logger.warning(f"Text field does not exist in TextNode at {node_path or '<root>'}")
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

    def __init__(self, node_dict: dict, node_path: str = "") -> None:
        super().__init__(node_dict, node_path)

        self._elements: list[Node] = []
        for child_node in self._child_nodes:
            # Ensure we have only list items as children
            if child_node.type not in (NodeType.LIST_ITEM, NodeType.TASK_ITEM):
                logger.warning(
                    f"Expected LIST_ITEM or TASK_ITEM under list node at {node_path or '<root>'}, "
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

        def has_header_cells(node: Node | None) -> bool:
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
    def language(self) -> str | None:
        """Get the programming language of the code block."""
        return self._attrs.get("language")


class InlineCardNode(Node):
    """Represents an inline card (link preview)."""

    @property
    def url(self) -> str | None:
        """Get the URL of the inline card."""
        return self._attrs.get("url")

    @property
    def data(self) -> str | None:
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


class EmojiNode(Node):
    """Represents an emoji."""

    @property
    def short_name(self) -> str:
        """Get the emoji short name (e.g., ':grinning:')."""
        return self._attrs.get("shortName", "")

    @property
    def emoji_id(self) -> str | None:
        """Get the emoji service ID."""
        return self._attrs.get("id")

    @property
    def text(self) -> str | None:
        """Get the text representation of the emoji (unicode character)."""
        return self._attrs.get("text")


# Node registry for factory pattern
_NODE_REGISTRY: dict[NodeType, type[Node]] = {
    NodeType.DOC: DocNode,
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
    NodeType.EMOJI: EmojiNode,
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
    "embedCard",
}


def create_node_from_dict(node_dict: dict, node_path: str = "") -> Node | None:
    """
    Create a node from a dictionary using registry pattern.

    Args:
        node_dict: Dictionary containing node data
        node_path: Path to this node in the ADF tree (for error reporting)

    Returns:
        Node instance

    Raises:
        UnsupportedNodeTypeError: If node type is not supported
        MissingFieldError: If required fields are missing
        NodeCreationError: If node creation fails
    """
    if "type" not in node_dict:
        raise MissingFieldError(
            field_name="type",
            node_path=node_path or "<root>",
            expected_values=NodeType.supported_values(),
        )

    node_type_str = node_dict["type"]
    current_path = f"{node_path} > {node_type_str}" if node_path else node_type_str

    # Handle known unsupported types gracefully
    if node_type_str in _KNOWN_UNSUPPORTED_TYPES:
        logger.debug(f"Skipping known unsupported node type: {node_type_str} at {current_path}")
        return Node(node_dict, node_path)

    try:
        node_type = NodeType.from_string(node_type_str)
    except InvalidFieldError as e:
        # Re-raise with proper node path and supported types
        logger.error(f"Unknown node type: {node_type_str} at {current_path}")
        raise UnsupportedNodeTypeError(
            node_type=node_type_str,
            node_path=current_path,
            supported_types=NodeType.supported_values(),
        ) from e

    # Get the appropriate node class from registry
    node_class = _NODE_REGISTRY.get(node_type)

    if node_class is None:
        logger.error(f"No handler registered for node type: {node_type} at {current_path}")
        raise UnsupportedNodeTypeError(
            node_type=node_type_str,
            node_path=current_path,
            supported_types=[str(k) for k in _NODE_REGISTRY.keys()],
        )

    try:
        return node_class(node_dict, node_path)
    except (MissingFieldError, InvalidFieldError, UnsupportedNodeTypeError):
        # These are already properly formatted, just re-raise
        raise
    except Exception as e:
        logger.error(f"Error creating node of type {node_type} at {current_path}: {e}")
        raise NodeCreationError(
            node_type=node_type_str,
            reason=str(e),
            node_path=current_path,
            original_error=e,
        ) from e


