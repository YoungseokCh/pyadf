"""Entry point: python -m benchmarks"""

from .bench_baseline import run_baseline
from .bench_jsonl import run as run_jsonl
from .bench_single import run as run_single
from .fixtures import make_rich_doc
from .runner import print_banner, print_header


def main() -> None:
    n_single = 10_000
    n_batch = 100_000

    print_banner("pyadf benchmark: PyPI (pure Python) vs Rust")

    # --- Baseline (PyPI) ---
    print_header("PyPI version (pure Python, v0.3.1)")
    dicts = [make_rich_doc(i) for i in range(n_batch)]
    baseline = run_baseline(dicts)
    if baseline:
        baseline.print()

    # --- Rust ---
    single = run_single(n=n_single)
    jsonl = run_jsonl(n=n_batch)

    # --- Comparison ---
    if baseline:
        print_header("Comparison")
        doc_rate = single["document_class"].rate
        direct_rate = single["core_direct"].rate
        batch_rate = jsonl.rate
        print(f"  Single doc speedup (Document class): {doc_rate / baseline.single_rate:.1f}x")
        print(f"  Single doc speedup (direct Rust):    {direct_rate / baseline.single_rate:.1f}x")
        print(f"  Batch speedup (JSONL vs sequential): {batch_rate / baseline.batch_rate:.1f}x")
    print()


if __name__ == "__main__":
    main()
