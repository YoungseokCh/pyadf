use std::fmt;

/// All error types that can occur during ADF parsing and rendering.
#[derive(Debug)]
pub enum AdfError {
    /// JSON parsing failed.
    InvalidJson {
        message: String,
        position: Option<usize>,
    },
    /// Input type is wrong (not a JSON object).
    InvalidInput {
        expected_type: String,
        actual_type: String,
    },
    /// A required field is missing from a node.
    MissingField {
        field_name: String,
        node_type: Option<String>,
        node_path: Option<String>,
        expected_values: Option<Vec<String>>,
    },
    /// A field has an invalid value.
    InvalidField {
        field_name: String,
        invalid_value: String,
        node_type: Option<String>,
        node_path: Option<String>,
        expected_values: Option<Vec<String>>,
    },
    /// An unsupported node type was encountered.
    UnsupportedNodeType {
        node_type: String,
        node_path: Option<String>,
        supported_types: Option<Vec<String>>,
    },
    /// Invalid configuration.
    InvalidConfig { message: String },
    /// Markdown parsing failed or hit unsupported syntax.
    MarkdownParse { message: String },
}

impl fmt::Display for AdfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdfError::InvalidJson { message, position } => {
                write!(f, "Invalid JSON: {message}")?;
                if let Some(pos) = position {
                    write!(f, " at position {pos}")?;
                }
                Ok(())
            }
            AdfError::InvalidInput {
                expected_type,
                actual_type,
            } => {
                write!(
                    f,
                    "Invalid input type: expected {expected_type}, got {actual_type}"
                )?;
                write!(f, "\n  Hint: Document() accepts JSON string, dict, or None")
            }
            AdfError::MissingField {
                field_name,
                node_type,
                node_path,
                expected_values,
            } => {
                write!(f, "Missing required field \"{field_name}\"")?;
                if let Some(nt) = node_type {
                    write!(f, " in node type \"{nt}\"")?;
                }
                if let Some(vals) = expected_values {
                    write_expected_values(f, vals)?;
                }
                if let Some(path) = node_path {
                    write!(f, "\n  at: {path}")?;
                }
                Ok(())
            }
            AdfError::InvalidField {
                field_name,
                invalid_value,
                node_type,
                node_path,
                expected_values,
            } => {
                write!(
                    f,
                    "Invalid value \"{invalid_value}\" for field \"{field_name}\""
                )?;
                if let Some(nt) = node_type {
                    write!(f, " in node type \"{nt}\"")?;
                }
                if let Some(vals) = expected_values {
                    write_expected_values(f, vals)?;
                }
                if let Some(path) = node_path {
                    write!(f, "\n  at: {path}")?;
                }
                Ok(())
            }
            AdfError::UnsupportedNodeType {
                node_type,
                node_path,
                supported_types,
            } => {
                write!(f, "Unsupported node type \"{node_type}\"")?;
                if let Some(types) = supported_types {
                    let limit = 15;
                    let mut sorted = types.clone();
                    sorted.sort();
                    if sorted.len() <= limit {
                        let s: Vec<String> = sorted.iter().map(|t| format!("\"{t}\"")).collect();
                        write!(f, "\n  Supported types: {}", s.join(", "))?;
                    } else {
                        let s: Vec<String> =
                            sorted[..limit].iter().map(|t| format!("\"{t}\"")).collect();
                        write!(
                            f,
                            "\n  Supported types: {}, ... ({} total)",
                            s.join(", "),
                            sorted.len()
                        )?;
                    }
                }
                if let Some(path) = node_path {
                    write!(f, "\n  at: {path}")?;
                }
                Ok(())
            }
            AdfError::InvalidConfig { message } => {
                write!(f, "{message}")
            }
            AdfError::MarkdownParse { message } => {
                write!(f, "Markdown parse error: {message}")
            }
        }
    }
}

fn write_expected_values(f: &mut fmt::Formatter<'_>, vals: &[String]) -> fmt::Result {
    let limit = 10;
    if vals.len() <= limit {
        let s: Vec<String> = vals.iter().map(|v| format!("\"{v}\"")).collect();
        write!(f, "\n  Expected one of: {}", s.join(", "))
    } else {
        let s: Vec<String> = vals[..limit].iter().map(|v| format!("\"{v}\"")).collect();
        write!(
            f,
            "\n  Expected one of: {}, ... ({} total)",
            s.join(", "),
            vals.len()
        )
    }
}

/// Convert an AdfError to a PyErr, raising the matching Python exception from pyadf.exceptions.
/// Caller must hold the GIL and pass `py`.
pub fn to_py_err(py: pyo3::Python<'_>, err: &AdfError) -> pyo3::PyErr {
    use pyo3::types::PyAnyMethods;

    let exceptions = match py.import("pyadf.exceptions") {
        Ok(m) => m,
        Err(_) => return pyo3::exceptions::PyValueError::new_err(err.to_string()),
    };

    match err {
        AdfError::InvalidJson { message, position } => {
            let cls = exceptions.getattr("InvalidJSONError").unwrap();
            match cls.call1((message.as_str(), *position)) {
                Ok(inst) => pyo3::PyErr::from_value(inst.into_any()),
                Err(_) => pyo3::exceptions::PyValueError::new_err(err.to_string()),
            }
        }
        AdfError::InvalidInput {
            expected_type,
            actual_type,
        } => {
            let cls = exceptions.getattr("InvalidInputError").unwrap();
            match cls.call1((expected_type.as_str(), actual_type.as_str())) {
                Ok(inst) => pyo3::PyErr::from_value(inst.into_any()),
                Err(_) => pyo3::exceptions::PyValueError::new_err(err.to_string()),
            }
        }
        AdfError::MissingField {
            field_name,
            node_type,
            node_path,
            expected_values,
        } => {
            let cls = exceptions.getattr("MissingFieldError").unwrap();
            let kwargs = pyo3::types::PyDict::new(py);
            let _ = kwargs.set_item("field_name", field_name.as_str());
            let _ = kwargs.set_item("node_type", node_type.as_deref());
            let _ = kwargs.set_item("node_path", node_path.as_deref());
            let _ = kwargs.set_item("expected_values", expected_values.as_ref());
            match cls.call((), Some(&kwargs)) {
                Ok(inst) => pyo3::PyErr::from_value(inst.into_any()),
                Err(_) => pyo3::exceptions::PyValueError::new_err(err.to_string()),
            }
        }
        AdfError::InvalidField {
            field_name,
            invalid_value,
            node_type,
            node_path,
            expected_values,
        } => {
            let cls = exceptions.getattr("InvalidFieldError").unwrap();
            let kwargs = pyo3::types::PyDict::new(py);
            let _ = kwargs.set_item("field_name", field_name.as_str());
            let _ = kwargs.set_item("invalid_value", invalid_value.as_str());
            let _ = kwargs.set_item("node_type", node_type.as_deref());
            let _ = kwargs.set_item("node_path", node_path.as_deref());
            let _ = kwargs.set_item("expected_values", expected_values.as_ref());
            match cls.call((), Some(&kwargs)) {
                Ok(inst) => pyo3::PyErr::from_value(inst.into_any()),
                Err(_) => pyo3::exceptions::PyValueError::new_err(err.to_string()),
            }
        }
        AdfError::UnsupportedNodeType {
            node_type,
            node_path,
            supported_types,
        } => {
            let cls = exceptions.getattr("UnsupportedNodeTypeError").unwrap();
            let kwargs = pyo3::types::PyDict::new(py);
            let _ = kwargs.set_item("node_type", node_type.as_str());
            let _ = kwargs.set_item("node_path", node_path.as_deref());
            let _ = kwargs.set_item("supported_types", supported_types.as_ref());
            match cls.call((), Some(&kwargs)) {
                Ok(inst) => pyo3::PyErr::from_value(inst.into_any()),
                Err(_) => pyo3::exceptions::PyValueError::new_err(err.to_string()),
            }
        }
        AdfError::MarkdownParse { message } => {
            let cls = exceptions.getattr("MarkdownParseError").unwrap();
            match cls.call1((message.as_str(),)) {
                Ok(inst) => pyo3::PyErr::from_value(inst.into_any()),
                Err(_) => pyo3::exceptions::PyValueError::new_err(err.to_string()),
            }
        }
        _ => pyo3::exceptions::PyValueError::new_err(err.to_string()),
    }
}
