"""Exception classes for pyadf with detailed error context."""



class PyADFError(Exception):
    """Base exception for all pyadf errors."""

    def __init__(self, message: str, node_path: str | None = None) -> None:
        """
        Initialize the exception.

        Args:
            message: Human-readable error message
            node_path: Optional path showing location in ADF tree
                (e.g., "doc > paragraph[0] > text")
        """
        self.message = message
        self.node_path = node_path
        super().__init__(self._format_message())

    def _format_message(self) -> str:
        """Format the error message with optional node path."""
        if self.node_path:
            return f"{self.message}\n  at: {self.node_path}"
        return self.message


class InvalidADFError(PyADFError):
    """Raised when ADF structure is invalid or malformed."""

    pass


class MissingFieldError(InvalidADFError):
    """Raised when a required field is missing from a node."""

    def __init__(
        self,
        field_name: str,
        node_type: str | None = None,
        node_path: str | None = None,
        expected_values: list[str] | None = None,
    ) -> None:
        """
        Initialize missing field error.

        Args:
            field_name: Name of the missing field
            node_type: Type of node where field is missing
            node_path: Path to the node in ADF tree
            expected_values: Optional list of valid values for the field
        """
        self.field_name = field_name
        self.node_type = node_type
        self.expected_values = expected_values

        message = f'Missing required field "{field_name}"'
        if node_type:
            message += f' in node type "{node_type}"'

        if expected_values:
            if len(expected_values) <= 10:
                values_str = ", ".join(f'"{v}"' for v in expected_values)
                message += f"\n  Expected one of: {values_str}"
            else:
                values_str = ", ".join(f'"{v}"' for v in expected_values[:10])
                message += f"\n  Expected one of: {values_str}, ... ({len(expected_values)} total)"

        super().__init__(message, node_path)


class InvalidFieldError(InvalidADFError):
    """Raised when a field has an invalid value."""

    def __init__(
        self,
        field_name: str,
        invalid_value: str,
        node_type: str | None = None,
        node_path: str | None = None,
        expected_values: list[str] | None = None,
    ) -> None:
        """
        Initialize invalid field error.

        Args:
            field_name: Name of the field with invalid value
            invalid_value: The invalid value that was provided
            node_type: Type of node containing the field
            node_path: Path to the node in ADF tree
            expected_values: Optional list of valid values for the field
        """
        self.field_name = field_name
        self.invalid_value = invalid_value
        self.node_type = node_type
        self.expected_values = expected_values

        message = f'Invalid value "{invalid_value}" for field "{field_name}"'
        if node_type:
            message += f' in node type "{node_type}"'

        if expected_values:
            if len(expected_values) <= 10:
                values_str = ", ".join(f'"{v}"' for v in expected_values)
                message += f"\n  Expected one of: {values_str}"
            else:
                values_str = ", ".join(f'"{v}"' for v in expected_values[:10])
                message += f"\n  Expected one of: {values_str}, ... ({len(expected_values)} total)"

        super().__init__(message, node_path)


class UnsupportedNodeTypeError(PyADFError):
    """Raised when an unsupported node type is encountered."""

    def __init__(
        self,
        node_type: str,
        node_path: str | None = None,
        supported_types: list[str] | None = None,
    ) -> None:
        """
        Initialize unsupported node type error.

        Args:
            node_type: The unsupported node type
            node_path: Path to the node in ADF tree
            supported_types: Optional list of supported node types
        """
        self.node_type = node_type
        self.supported_types = supported_types

        message = f'Unsupported node type "{node_type}"'

        if supported_types:
            if len(supported_types) <= 15:
                types_str = ", ".join(f'"{t}"' for t in sorted(supported_types))
                message += f"\n  Supported types: {types_str}"
            else:
                types_str = ", ".join(f'"{t}"' for t in sorted(supported_types)[:15])
                message += f"\n  Supported types: {types_str}, ... ({len(supported_types)} total)"

        super().__init__(message, node_path)


class InvalidJSONError(InvalidADFError):
    """Raised when ADF JSON cannot be parsed."""

    def __init__(self, json_error: str, position: int | None = None) -> None:
        """
        Initialize invalid JSON error.

        Args:
            json_error: The JSON parsing error message
            position: Optional character position where error occurred
        """
        self.json_error = json_error
        self.position = position

        message = f"Invalid JSON: {json_error}"
        if position is not None:
            message += f" at position {position}"

        super().__init__(message)


class NodeCreationError(PyADFError):
    """Raised when a node cannot be created from data."""

    def __init__(
        self,
        node_type: str,
        reason: str,
        node_path: str | None = None,
        original_error: Exception | None = None,
    ) -> None:
        """
        Initialize node creation error.

        Args:
            node_type: Type of node that failed to create
            reason: Reason for the failure
            node_path: Path to the node in ADF tree
            original_error: Original exception that caused the failure
        """
        self.node_type = node_type
        self.reason = reason
        self.original_error = original_error

        message = f'Failed to create node of type "{node_type}": {reason}'

        if original_error:
            message += f"\n  Caused by: {type(original_error).__name__}: {str(original_error)}"

        super().__init__(message, node_path)


class InvalidInputError(PyADFError):
    """Raised when input to Document class is invalid."""

    def __init__(self, expected_type: str, actual_type: str) -> None:
        """
        Initialize invalid input error.

        Args:
            expected_type: Expected input type
            actual_type: Actual input type received
        """
        self.expected_type = expected_type
        self.actual_type = actual_type

        message = f"Invalid input type: expected {expected_type}, got {actual_type}"
        message += "\n  Hint: Document() accepts JSON string, dict, or None"

        super().__init__(message)
