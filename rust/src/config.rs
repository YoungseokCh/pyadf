use pyo3::prelude::*;

use crate::errors::AdfError;

/// Markdown rendering configuration (internal).
#[derive(Debug, Clone)]
pub struct MarkdownConfig {
    pub bullet_marker: String,
    pub show_links: bool,
}

impl MarkdownConfig {
    pub fn new(bullet_marker: &str, show_links: bool) -> Result<Self, AdfError> {
        match bullet_marker {
            "+" | "-" | "*" => Ok(Self {
                bullet_marker: bullet_marker.to_string(),
                show_links,
            }),
            _ => Err(AdfError::InvalidConfig {
                message: format!("Invalid bullet_marker: {bullet_marker:?}"),
            }),
        }
    }
}

impl Default for MarkdownConfig {
    fn default() -> Self {
        Self {
            bullet_marker: "+".to_string(),
            show_links: false,
        }
    }
}

/// Python-exposed markdown configuration.
#[pyclass(frozen)]
#[pyo3(name = "MarkdownConfig")]
#[derive(Debug, Clone)]
pub struct PyMarkdownConfig {
    #[pyo3(get)]
    pub bullet_marker: String,
    #[pyo3(get)]
    pub show_links: bool,
}

#[pymethods]
impl PyMarkdownConfig {
    #[new]
    #[pyo3(signature = (bullet_marker="+", show_links=false))]
    fn new(bullet_marker: &str, show_links: bool) -> PyResult<Self> {
        match bullet_marker {
            "+" | "-" | "*" => Ok(Self {
                bullet_marker: bullet_marker.to_string(),
                show_links,
            }),
            _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Invalid bullet_marker: {bullet_marker:?}"
            ))),
        }
    }
}

impl PyMarkdownConfig {
    pub fn to_internal(&self) -> MarkdownConfig {
        MarkdownConfig {
            bullet_marker: self.bullet_marker.clone(),
            show_links: self.show_links,
        }
    }
}
