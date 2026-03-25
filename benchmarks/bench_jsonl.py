"""Benchmark JSONL batch conversion."""

from __future__ import annotations

import time

from .fixtures import make_jsonl, make_rich_doc
from .runner import BenchResult, print_header


def run(n: int = 100_000) -> BenchResult:
    """Run JSONL batch benchmark and return the result."""
    from pyadf import convert_jsonl

    dicts = [make_rich_doc(i) for i in range(n)]
    data = make_jsonl(dicts)
    size_mb = len(data) / (1024 * 1024)

    print_header(f"JSONL batch conversion (n={n:,})")

    # Warmup
    small = make_jsonl(dicts[:100])
    list(convert_jsonl(small, batch_size=100))

    start = time.perf_counter()
    list(convert_jsonl(data, batch_size=10_000))
    elapsed = time.perf_counter() - start

    rate = n / elapsed
    throughput = size_mb / elapsed
    result = BenchResult(label="Rust JSONL (rayon parallel)", n=n, elapsed=elapsed)
    print(
        f"  {result.label:40s} {n:>8,} iters in {elapsed:.3f}s"
        f"  →  {rate:>10,.0f} docs/s  ({throughput:.0f} MB/s)"
    )
    return result
