"""Sphinx configuration for pyadf documentation."""

project = "pyadf"
copyright = "2024, Youngseok Choi"
author = "Youngseok Choi"
release = "0.5.0"

extensions = [
    "sphinx.ext.autodoc",
    "sphinx.ext.napoleon",
    "sphinx.ext.intersphinx",
    "sphinx.ext.viewcode",
    "sphinx_copybutton",
    "myst_parser",
]

templates_path = ["_templates"]
exclude_patterns = ["_build"]

html_theme = "furo"
html_title = "pyadf"
html_theme_options = {
    "source_repository": "https://github.com/YoungseokCh/pyadf",
    "source_branch": "main",
    "source_directory": "docs/",
}

autodoc_member_order = "bysource"
autodoc_typehints = "description"

napoleon_google_docstyle = True
napoleon_numpy_docstyle = False

intersphinx_mapping = {
    "python": ("https://docs.python.org/3", None),
}

myst_enable_extensions = [
    "colon_fence",
    "fieldlist",
]
