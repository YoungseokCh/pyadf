"""Main converter function from ADF to Markdown."""

from . import markdown, nodes


def adf2md(json_data: dict) -> str:
    """
    Convert Atlassian Document Format (ADF) to Markdown.

    Args:
        json_data: ADF data as a dict.
                   Can be any ADF node type including "doc".

    Returns:
        Markdown representation of the ADF content

    Raises:
        ValueError: If json_data is not a dict
        nodes.UnsupportedNodeTypeError: If ADF contains unsupported node types
        nodes.InvalidNodeError: If ADF data is malformed
    """
    if not isinstance(json_data, dict):
        raise ValueError(f"Expected dict, got {type(json_data)}")

    # Create node from the dict (can be any node type)
    root_node = nodes.create_node_from_dict(json_data)
    if root_node is None:
        return ""

    return markdown.gen_md_from_root_node(root_node)
