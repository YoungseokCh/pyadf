"""Benchmark the published PyPI version (pure Python) in an isolated subprocess."""

from __future__ import annotations

import json
import subprocess
import tempfile
from dataclasses import dataclass
from pathlib import Path

# Script executed inside the isolated PyPI venv.
_BENCH_SCRIPT = """\
import json, time, sys

adf_json = sys.stdin.read()
adf_dicts = [json.loads(line) for line in adf_json.strip().split("\\n")]

from pyadf import Document

# Warmup
for d in adf_dicts[:100]:
    Document(d).to_markdown()

# Single-doc
doc = adf_dicts[0]
n_single = 10_000
start = time.perf_counter()
for _ in range(n_single):
    Document(doc).to_markdown()
single_elapsed = time.perf_counter() - start

# Batch (sequential)
n_batch = len(adf_dicts)
start = time.perf_counter()
for d in adf_dicts:
    Document(d).to_markdown()
batch_elapsed = time.perf_counter() - start

print(json.dumps({
    "single_rate": n_single / single_elapsed,
    "single_elapsed": single_elapsed,
    "batch_rate": n_batch / batch_elapsed,
    "batch_elapsed": batch_elapsed,
    "n_single": n_single,
    "n_batch": n_batch,
}))
"""


@dataclass
class BaselineResult:
    single_rate: float
    single_elapsed: float
    batch_rate: float
    batch_elapsed: float
    n_single: int
    n_batch: int

    def print(self, width: int = 40) -> None:
        print(
            f"  {'PyPI single doc':{width}s} {self.n_single:>8,} iters"
            f" in {self.single_elapsed:.3f}s  →  {self.single_rate:>10,.0f} docs/s"
        )
        print(
            f"  {'PyPI batch (sequential)':{width}s} {self.n_batch:>8,} iters"
            f" in {self.batch_elapsed:.3f}s  →  {self.batch_rate:>10,.0f} docs/s"
        )


def run_baseline(adf_dicts: list[dict], timeout: int = 120) -> BaselineResult | None:
    """Install pyadf from PyPI into a temp venv and benchmark it."""
    with tempfile.TemporaryDirectory() as tmpdir:
        venv_dir = Path(tmpdir) / "venv"
        print("  Setting up PyPI pyadf venv...", end=" ", flush=True)

        if _run(["uv", "venv", str(venv_dir)]).returncode != 0:
            print("FAILED (uv venv)")
            return None

        python = str(venv_dir / "bin" / "python")

        if _run(["uv", "pip", "install", "--python", python, "pyadf"]).returncode != 0:
            print("FAILED (install)")
            return None
        print("OK")

        jsonl_input = "\n".join(json.dumps(d) for d in adf_dicts)

        print("  Running PyPI benchmark...", end=" ", flush=True)
        result = subprocess.run(
            [python, "-c", _BENCH_SCRIPT],
            input=jsonl_input,
            capture_output=True,
            text=True,
            timeout=timeout,
        )
        if result.returncode != 0:
            print(f"FAILED ({result.stderr[:200]})")
            return None

        data = json.loads(result.stdout.strip())
        print("OK")
        return BaselineResult(**data)


def _run(cmd: list[str]) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(cmd, capture_output=True)
