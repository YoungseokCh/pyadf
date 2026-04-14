use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyNone, PyString};

mod adf_node;
mod config;
mod errors;
mod markdown;

/// Convert a Python dict/list/scalar to serde_json::Value without going through JSON text.
fn py_to_json_value(obj: &Bound<'_, PyAny>) -> PyResult<serde_json::Value> {
    if obj.is_none() || obj.is_instance_of::<PyNone>() {
        Ok(serde_json::Value::Null)
    } else if let Ok(b) = obj.downcast::<PyBool>() {
        Ok(serde_json::Value::Bool(b.is_true()))
    } else if let Ok(i) = obj.downcast::<PyInt>() {
        let v: i64 = i.extract()?;
        Ok(serde_json::Value::Number(v.into()))
    } else if let Ok(f) = obj.downcast::<PyFloat>() {
        let v: f64 = f.extract()?;
        match serde_json::Number::from_f64(v) {
            Some(n) => Ok(serde_json::Value::Number(n)),
            None => Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Non-finite float value ({v}) cannot be represented in JSON"
            ))),
        }
    } else if let Ok(s) = obj.downcast::<PyString>() {
        Ok(serde_json::Value::String(s.to_string()))
    } else if let Ok(list) = obj.downcast::<PyList>() {
        let mut arr = Vec::with_capacity(list.len());
        for item in list.iter() {
            arr.push(py_to_json_value(&item)?);
        }
        Ok(serde_json::Value::Array(arr))
    } else if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut map = serde_json::Map::with_capacity(dict.len());
        for (k, v) in dict.iter() {
            let key: String = k.extract()?;
            map.insert(key, py_to_json_value(&v)?);
        }
        Ok(serde_json::Value::Object(map))
    } else {
        let type_name: String = obj.get_type().name()?.extract()?;
        Err(pyo3::exceptions::PyTypeError::new_err(format!(
            "Cannot convert Python type '{type_name}' to JSON value"
        )))
    }
}

// ---------------------------------------------------------------------------
// Opaque handle: parse once in __init__, render from cache in to_markdown()
// ---------------------------------------------------------------------------

/// Parsed ADF tree held as a Python object. Parse errors surface at creation
/// time (input validation), rendering from the cached tree cannot fail due to
/// bad input — only logic bugs.
#[pyclass(frozen)]
struct ParsedAdf {
    node: adf_node::AdfNode,
    skipped_nodes: Vec<adf_node::SkippedNode>,
}

type SkippedNodeInfo = (String, String);
type BatchResult = (Option<String>, Option<String>, Vec<SkippedNodeInfo>);

fn parse_known_unsupported_mode(mode: &str) -> PyResult<adf_node::KnownUnsupportedMode> {
    match mode {
        "error" => Ok(adf_node::KnownUnsupportedMode::Error),
        "skip" => Ok(adf_node::KnownUnsupportedMode::Skip),
        "warn" => Ok(adf_node::KnownUnsupportedMode::Warn),
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "on_known_unsupported must be 'error', 'skip', or 'warn', got {mode:?}"
        ))),
    }
}

/// Parse an ADF JSON string and return a cached handle.
#[pyfunction]
#[pyo3(signature = (json, on_known_unsupported="warn"))]
fn parse_adf_str(py: Python<'_>, json: &str, on_known_unsupported: &str) -> PyResult<ParsedAdf> {
    let mode = parse_known_unsupported_mode(on_known_unsupported)?;
    let parsed = adf_node::parse_adf(json, mode).map_err(|e| errors::to_py_err(py, &e))?;
    Ok(ParsedAdf {
        node: parsed.node,
        skipped_nodes: parsed.skipped_nodes,
    })
}

/// Parse a Python dict and return a cached handle (no JSON round-trip).
#[pyfunction]
#[pyo3(signature = (adf_dict, on_known_unsupported="warn"))]
fn parse_adf_dict(
    py: Python<'_>,
    adf_dict: &Bound<'_, PyAny>,
    on_known_unsupported: &str,
) -> PyResult<ParsedAdf> {
    let value = py_to_json_value(adf_dict)?;
    let mode = parse_known_unsupported_mode(on_known_unsupported)?;
    let parsed =
        adf_node::parse_adf_value(&value, "", mode).map_err(|e| errors::to_py_err(py, &e))?;
    Ok(ParsedAdf {
        node: parsed.node,
        skipped_nodes: parsed.skipped_nodes,
    })
}

#[pyfunction]
fn skipped_known_unsupported(parsed: &ParsedAdf) -> Vec<(String, String)> {
    parsed
        .skipped_nodes
        .iter()
        .map(|node| (node.node_type.clone(), node.node_path.clone()))
        .collect()
}

/// Render a previously parsed ADF tree to markdown.
#[pyfunction]
#[pyo3(signature = (parsed, config=None))]
fn render_markdown(
    parsed: &ParsedAdf,
    config: Option<&config::PyMarkdownConfig>,
) -> PyResult<String> {
    let cfg = match config {
        Some(c) => c.to_internal(),
        None => config::MarkdownConfig::default(),
    };
    Ok(markdown::render(&parsed.node, &cfg))
}

// ---------------------------------------------------------------------------
// One-shot convenience function (parse + render in single call, for JSONL)
// ---------------------------------------------------------------------------

/// Convert an ADF JSON string to markdown in one shot.
#[pyfunction]
#[pyo3(signature = (json, config=None, on_known_unsupported="warn"))]
fn document_to_markdown(
    py: Python<'_>,
    json: &str,
    config: Option<&config::PyMarkdownConfig>,
    on_known_unsupported: &str,
) -> PyResult<String> {
    let cfg = match config {
        Some(c) => c.to_internal(),
        None => config::MarkdownConfig::default(),
    };
    let mode = parse_known_unsupported_mode(on_known_unsupported)?;
    let parsed = adf_node::parse_adf(json, mode).map_err(|e| errors::to_py_err(py, &e))?;
    Ok(markdown::render(&parsed.node, &cfg))
}

// ---------------------------------------------------------------------------
// JSONL batch processing
// ---------------------------------------------------------------------------

/// Process a JSONL batch: takes bytes, returns list of (markdown_or_none, error_or_none) tuples.
///
/// NOTE: Uses rayon's global thread pool with py.allow_threads(). Safe for normal Python
/// processes. If used with multiprocessing fork-mode workers, rayon threads may inherit
/// unexpected state. Prefer spawn-mode workers (`multiprocessing.set_start_method("spawn")`).
#[pyfunction]
#[pyo3(signature = (data, config=None, on_known_unsupported="warn"))]
fn convert_jsonl_batch(
    py: Python<'_>,
    data: &[u8],
    config: Option<&config::PyMarkdownConfig>,
    on_known_unsupported: &str,
) -> PyResult<Vec<BatchResult>> {
    let cfg = match config {
        Some(c) => c.to_internal(),
        None => config::MarkdownConfig::default(),
    };
    let mode = parse_known_unsupported_mode(on_known_unsupported)?;

    // Process lines in parallel. Empty lines produce None and are stripped,
    // preserving 1:1 correspondence with the Python side's non_blank_lines.
    let results: Vec<BatchResult> = py.allow_threads(|| {
        use rayon::prelude::*;
        let lines: Vec<&[u8]> = data.split(|&b| b == b'\n').collect();
        lines
            .par_iter()
            .filter_map(|line| {
                if line.is_empty() {
                    return None;
                }
                let json = match std::str::from_utf8(line) {
                    Ok(s) => s,
                    Err(e) => return Some((None, Some(e.to_string()), Vec::new())),
                };
                match adf_node::parse_adf(json, mode) {
                    Ok(parsed) => Some((
                        Some(markdown::render(&parsed.node, &cfg)),
                        None,
                        parsed
                            .skipped_nodes
                            .into_iter()
                            .map(|node| (node.node_type, node.node_path))
                            .collect(),
                    )),
                    Err(e) => Some((None, Some(e.to_string()), Vec::new())),
                }
            })
            .collect()
    });

    Ok(results)
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ParsedAdf>()?;
    m.add_class::<config::PyMarkdownConfig>()?;
    m.add_function(wrap_pyfunction!(parse_adf_str, m)?)?;
    m.add_function(wrap_pyfunction!(parse_adf_dict, m)?)?;
    m.add_function(wrap_pyfunction!(skipped_known_unsupported, m)?)?;
    m.add_function(wrap_pyfunction!(render_markdown, m)?)?;
    m.add_function(wrap_pyfunction!(document_to_markdown, m)?)?;
    m.add_function(wrap_pyfunction!(convert_jsonl_batch, m)?)?;
    Ok(())
}
