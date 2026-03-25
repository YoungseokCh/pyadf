"""Benchmark harness: timing, warmup, and reporting."""

from __future__ import annotations

import time
from collections.abc import Callable
from dataclasses import dataclass
from typing import Any


@dataclass
class BenchResult:
    label: str
    n: int
    elapsed: float

    @property
    def rate(self) -> float:
        return self.n / self.elapsed

    def print(self, width: int = 40) -> None:
        print(
            f"  {self.label:{width}s} {self.n:>8,} iters in {self.elapsed:.3f}s"
            f"  →  {self.rate:>10,.0f} docs/s"
        )


def bench(
    label: str,
    fn: Callable[..., Any],
    args: tuple[Any, ...] = (),
    n: int = 10_000,
    warmup: int = 100,
) -> BenchResult:
    """Time *fn(*args)* for *n* iterations after *warmup* calls."""
    for _ in range(min(warmup, n)):
        fn(*args)

    start = time.perf_counter()
    for _ in range(n):
        fn(*args)
    elapsed = time.perf_counter() - start

    result = BenchResult(label=label, n=n, elapsed=elapsed)
    result.print()
    return result


def print_header(title: str) -> None:
    print()
    print(title)
    print("-" * 75)


def print_banner(title: str) -> None:
    print("=" * 75)
    print(title)
    print("=" * 75)
