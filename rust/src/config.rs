use chrono::format::{Item, StrftimeItems};
use chrono_tz::Tz;
use pyo3::prelude::*;

use crate::errors::AdfError;

/// Markdown rendering configuration (internal).
#[derive(Debug, Clone)]
pub struct MarkdownConfig {
    pub bullet_marker: String,
    pub show_links: bool,
    pub date_format: String,
    pub date_timezone: String,
}

impl MarkdownConfig {
    pub fn new(
        bullet_marker: &str,
        show_links: bool,
        date_timezone: &str,
        date_format: &str,
    ) -> Result<Self, AdfError> {
        validate_bullet_marker(bullet_marker)
            .map_err(|message| AdfError::InvalidConfig { message })?;
        validate_timezone(date_timezone).map_err(|message| AdfError::InvalidConfig { message })?;
        validate_date_format(date_format).map_err(|message| AdfError::InvalidConfig { message })?;
        Ok(Self {
            bullet_marker: bullet_marker.to_string(),
            show_links,
            date_format: date_format.to_string(),
            date_timezone: date_timezone.to_string(),
        })
    }
}

impl Default for MarkdownConfig {
    fn default() -> Self {
        Self {
            bullet_marker: "-".to_string(),
            show_links: true,
            date_format: "%Y-%m-%dT%H:%M:%S%:z".to_string(),
            date_timezone: "UTC".to_string(),
        }
    }
}

fn validate_bullet_marker(bullet_marker: &str) -> Result<(), String> {
    match bullet_marker {
        "+" | "-" | "*" => Ok(()),
        _ => Err(format!("Invalid bullet_marker: {bullet_marker:?}")),
    }
}

fn validate_timezone(timezone: &str) -> Result<(), String> {
    timezone
        .parse::<Tz>()
        .map(|_| ())
        .map_err(|_| format!("Invalid date_timezone: {timezone:?}"))
}

fn validate_date_format(format: &str) -> Result<(), String> {
    if StrftimeItems::new(format).any(|item| matches!(item, Item::Error)) {
        return Err(format!("Invalid date_format: {format:?}"));
    }
    Ok(())
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
    #[pyo3(get)]
    pub date_timezone: String,
    #[pyo3(get)]
    pub date_format: String,
}

#[pymethods]
impl PyMarkdownConfig {
    #[new]
    #[pyo3(signature = (
        bullet_marker="-",
        show_links=true,
        date_timezone="UTC",
        date_format="%Y-%m-%dT%H:%M:%S%:z",
    ))]
    fn new(
        bullet_marker: &str,
        show_links: bool,
        date_timezone: &str,
        date_format: &str,
    ) -> PyResult<Self> {
        validate_bullet_marker(bullet_marker).map_err(pyo3::exceptions::PyValueError::new_err)?;
        validate_timezone(date_timezone).map_err(pyo3::exceptions::PyValueError::new_err)?;
        validate_date_format(date_format).map_err(pyo3::exceptions::PyValueError::new_err)?;
        Ok(Self {
            bullet_marker: bullet_marker.to_string(),
            show_links,
            date_timezone: date_timezone.to_string(),
            date_format: date_format.to_string(),
        })
    }
}

impl PyMarkdownConfig {
    pub fn to_internal(&self) -> MarkdownConfig {
        MarkdownConfig {
            bullet_marker: self.bullet_marker.clone(),
            show_links: self.show_links,
            date_format: self.date_format.clone(),
            date_timezone: self.date_timezone.clone(),
        }
    }
}
