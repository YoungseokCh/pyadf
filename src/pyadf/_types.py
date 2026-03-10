"""Internal type aliases for parsed JSON and supported ADF shapes."""

from typing import Literal, NotRequired, TypeAlias, TypedDict

JSONScalar: TypeAlias = None | bool | int | float | str
JSONObject: TypeAlias = dict[str, "JSONValue"]
JSONArray: TypeAlias = list["JSONValue"]
JSONValue: TypeAlias = JSONScalar | JSONObject | JSONArray
BulletMarker: TypeAlias = Literal["+", "-", "*"]


class MarkDict(TypedDict, total=False):
    """Supported text mark fields used by this library."""

    type: str


class ADFNodeDict(TypedDict):
    """Minimal recursive ADF node shape used by the parser."""

    type: str
    attrs: NotRequired[JSONObject]
    content: NotRequired[list["ADFNodeDict"]]
    text: NotRequired[str]
    marks: NotRequired[list[MarkDict]]
