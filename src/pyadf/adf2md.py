"""Main converter function from ADF to Markdown."""

from typing import Union

from . import markdown, nodes


def adf2md(json_data: Union[dict, list[dict], None]) -> str:
    """
    Convert Atlassian Document Format (ADF) to Markdown.

    Args:
        json_data: ADF data as a dict, list of dicts, or None.
                   Can be a full document with type="doc", or individual nodes.

    Returns:
        Markdown representation of the ADF content

    Raises:
        ValueError: If json_data is of an unexpected type
        nodes.UnsupportedNodeTypeError: If ADF contains unsupported node types
        nodes.InvalidNodeError: If ADF data is malformed
    """
    root_nodes = []

    if isinstance(json_data, list):
        root_nodes = nodes.create_nodes_from_list(json_data)
    elif isinstance(json_data, dict):
        # Only extract content for document-level nodes (like "doc"),
        # not for content nodes like blockquote, panel, etc.
        if (
            "type" in json_data
            and json_data["type"] == "doc"
            and "content" in json_data
            and isinstance(json_data["content"], list)
        ):
            root_nodes = nodes.create_nodes_from_list(json_data["content"])
        else:
            root_node = nodes.create_node_from_dict(json_data)
            if root_node:
                root_nodes.append(root_node)
    elif json_data is None:
        return ""
    else:
        raise ValueError(f"Unexpected type: {type(json_data)}")

    md_text_list = [markdown.gen_md_from_root_node(node) for node in root_nodes]

    # Filter out empty strings
    md_text_list = [text for text in md_text_list if text]

    if not md_text_list:
        return ""

    return "\n\n".join(md_text_list)
