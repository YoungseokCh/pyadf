use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyNone, PyString};

mod adf_node;
mod adf_serialize;
mod config;
mod errors;
mod html_render;
mod html_to_adf;
mod jira_markup;
mod jira_patterns;
mod jira_render;
mod jira_to_adf;
mod markdown;
mod md_inline;
mod md_to_adf;
mod node_builders;

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
}

/// Parse an ADF JSON string and return a cached handle.
#[pyfunction]
fn parse_adf_str(py: Python<'_>, json: &str) -> PyResult<ParsedAdf> {
    let node = adf_node::parse_adf(json).map_err(|e| errors::to_py_err(py, &e))?;
    Ok(ParsedAdf { node })
}

/// Parse a Python dict and return a cached handle (no JSON round-trip).
#[pyfunction]
fn parse_adf_dict(py: Python<'_>, adf_dict: &Bound<'_, PyAny>) -> PyResult<ParsedAdf> {
    let value = py_to_json_value(adf_dict)?;
    let node = adf_node::parse_adf_value(&value, "").map_err(|e| errors::to_py_err(py, &e))?;
    Ok(ParsedAdf { node })
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
#[pyo3(signature = (json, config=None))]
fn document_to_markdown(
    py: Python<'_>,
    json: &str,
    config: Option<&config::PyMarkdownConfig>,
) -> PyResult<String> {
    let cfg = match config {
        Some(c) => c.to_internal(),
        None => config::MarkdownConfig::default(),
    };
    let node = adf_node::parse_adf(json).map_err(|e| errors::to_py_err(py, &e))?;
    Ok(markdown::render(&node, &cfg))
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
#[pyo3(signature = (data, config=None))]
fn convert_jsonl_batch(
    py: Python<'_>,
    data: &[u8],
    config: Option<&config::PyMarkdownConfig>,
) -> PyResult<Vec<(Option<String>, Option<String>)>> {
    let cfg = match config {
        Some(c) => c.to_internal(),
        None => config::MarkdownConfig::default(),
    };

    // Process lines in parallel. Empty lines produce None and are stripped,
    // preserving 1:1 correspondence with the Python side's non_blank_lines.
    let results: Vec<(Option<String>, Option<String>)> = py.allow_threads(|| {
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
                    Err(e) => return Some((None, Some(e.to_string()))),
                };
                match adf_node::parse_adf(json) {
                    Ok(node) => Some((Some(markdown::render(&node, &cfg)), None)),
                    Err(e) => Some((None, Some(e.to_string()))),
                }
            })
            .collect()
    });

    Ok(results)
}

// ---------------------------------------------------------------------------
// Output renderers: ADF JSON, HTML, Jira
// ---------------------------------------------------------------------------

/// Convert serde_json::Value to a Python object.
fn json_value_to_py(py: Python<'_>, value: &serde_json::Value) -> PyResult<PyObject> {
    match value {
        serde_json::Value::Null => Ok(py.None()),
        serde_json::Value::Bool(b) => Ok(PyBool::new(py, *b).to_owned().into_any().unbind()),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_pyobject(py)?.into_any().unbind())
            } else if let Some(f) = n.as_f64() {
                Ok(f.into_pyobject(py)?.into_any().unbind())
            } else {
                Ok(py.None())
            }
        }
        serde_json::Value::String(s) => Ok(s.into_pyobject(py)?.into_any().unbind()),
        serde_json::Value::Array(arr) => {
            let list = pyo3::types::PyList::empty(py);
            for item in arr {
                list.append(json_value_to_py(py, item)?)?;
            }
            Ok(list.into_any().unbind())
        }
        serde_json::Value::Object(map) => {
            let dict = pyo3::types::PyDict::new(py);
            for (k, v) in map {
                dict.set_item(k, json_value_to_py(py, v)?)?;
            }
            Ok(dict.into_any().unbind())
        }
    }
}

/// Serialize a parsed ADF tree to a Python dict (ADF JSON).
#[pyfunction]
fn render_adf_json(py: Python<'_>, parsed: &ParsedAdf) -> PyResult<PyObject> {
    let value = adf_serialize::serialize_to_value(&parsed.node);
    json_value_to_py(py, &value)
}

/// Render a parsed ADF tree to HTML.
#[pyfunction]
fn render_html(parsed: &ParsedAdf) -> String {
    html_render::render_html(&parsed.node)
}

/// Render a parsed ADF tree to Jira wiki markup.
#[pyfunction]
fn render_jira(parsed: &ParsedAdf) -> String {
    jira_render::render_jira(&parsed.node)
}

// ---------------------------------------------------------------------------
// Format-specific parsers
// ---------------------------------------------------------------------------

/// Parse Jira wiki markup and return a cached ADF handle.
#[pyfunction]
fn parse_jira_str(input: &str) -> ParsedAdf {
    ParsedAdf { node: jira_to_adf::parse_jira(input) }
}

/// Parse Markdown and return a cached ADF handle.
#[pyfunction]
fn parse_markdown_str(input: &str) -> ParsedAdf {
    ParsedAdf { node: md_to_adf::parse_markdown(input) }
}

/// Parse HTML/XHTML and return a cached ADF handle.
#[pyfunction]
fn parse_html_str(input: &str) -> ParsedAdf {
    ParsedAdf { node: html_to_adf::parse_html(input) }
}

#[pyfunction]
fn markdown_to_jira(input: &str) -> String {
    jira_markup::markdown_to_jira(input)
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ParsedAdf>()?;
    m.add_class::<config::PyMarkdownConfig>()?;
    m.add_function(wrap_pyfunction!(parse_adf_str, m)?)?;
    m.add_function(wrap_pyfunction!(parse_adf_dict, m)?)?;
    m.add_function(wrap_pyfunction!(render_markdown, m)?)?;
    m.add_function(wrap_pyfunction!(document_to_markdown, m)?)?;
    m.add_function(wrap_pyfunction!(convert_jsonl_batch, m)?)?;
    m.add_function(wrap_pyfunction!(parse_jira_str, m)?)?;
    m.add_function(wrap_pyfunction!(parse_markdown_str, m)?)?;
    m.add_function(wrap_pyfunction!(parse_html_str, m)?)?;
    m.add_function(wrap_pyfunction!(markdown_to_jira, m)?)?;
    m.add_function(wrap_pyfunction!(render_adf_json, m)?)?;
    m.add_function(wrap_pyfunction!(render_html, m)?)?;
    m.add_function(wrap_pyfunction!(render_jira, m)?)?;
    Ok(())
}
