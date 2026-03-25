"""Benchmark single-document conversion."""

from __future__ import annotations

import json

from .fixtures import make_rich_doc
from .runner import BenchResult, bench, print_header


def run(n: int = 10_000) -> dict[str, BenchResult]:
    """Run single-document benchmarks and return results keyed by label."""
    from pyadf import Document
    from pyadf._core import document_to_markdown

    adf_dict = make_rich_doc(0)
    adf_json = json.dumps(adf_dict)

    print_header(f"Single document conversion (n={n:,})")

    results: dict[str, BenchResult] = {}
    results["document_class"] = bench(
        "Rust via Document class",
        lambda d: Document(d).to_markdown(),
        (adf_dict,),
        n=n,
    )
    results["core_direct"] = bench(
        "Rust _core direct",
        document_to_markdown,
        (adf_json,),
        n=n,
    )
    return results
