#!/usr/bin/env python3
"""
Benchmark: swarm task decomposition with local models.

Compares single-agent vs multi-worker (swarm-style) execution on a
real coding task, using only local models (Ollama / OMLX).

Measures:
  - Wall-clock time per worker and total
  - RSS memory usage per process (via psutil or /proc)
  - Task decomposition quality (subtask coverage)
  - Result fusion success rate

Design note — ThreadPoolExecutor vs multiprocessing:
  This benchmark uses ThreadPoolExecutor for worker parallelism. Because Python's
  GIL means threads share the same process address space, RSS measurements reflect
  shared memory rather than per-process isolation. This is intentional: local LLM
  servers (Ollama, OMLX) are external processes — each worker makes an HTTP call
  to the same server, so the RAM cost is dominated by the server process, not the
  Python workers. If you need true per-process RAM isolation measurements, switch
  to multiprocessing (see the TODO comment in run_worker). For benchmarking API
  call throughput and wall-clock speedup, threads are the right choice.

Usage:
    python scripts/benchmark_local_swarm.py                    # defaults
    python scripts/benchmark_local_swarm.py --workers 10       # 10 workers
    python scripts/benchmark_local_swarm.py --model qwen2.5:7b # model name
    python scripts/benchmark_local_swarm.py --base-url http://localhost:11434/v1
    python scripts/benchmark_local_swarm.py --task-file tasks.json

Environment:
    BENCHMARK_BASE_URL  — OpenAI-compatible API base URL (default: http://localhost:11434/v1)
    BENCHMARK_MODEL     — model name (default: qwen2.5:7b)
    BENCHMARK_TIMEOUT   — per-request timeout in seconds (default: 300)

Requires: pip install openai psutil
"""

import argparse
import json
import os
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Optional

# ---------------------------------------------------------------------------
# Dependencies (fail fast with helpful message)
# ---------------------------------------------------------------------------

try:
    from openai import OpenAI
except ImportError:
    print("ERROR: 'openai' package required. Install with:")
    print("  pip install openai")
    sys.exit(1)

try:
    import psutil
except ImportError:
    print("WARNING: 'psutil' not installed — RAM measurement will be approximate.")
    print("  Install with: pip install psutil")
    psutil = None

# ---------------------------------------------------------------------------
# Data models
# ---------------------------------------------------------------------------


@dataclass
class WorkerResult:
    worker_id: str
    subtask_index: int
    status: str  # "success", "error", "timeout"
    elapsed_ms: float
    rss_mb: float  # peak RSS in MB
    output_length: int
    error: Optional[str] = None
    output: Optional[str] = None  # full worker output (for fusion)


@dataclass
class BenchmarkResult:
    task_description: str
    num_workers: int
    model: str
    base_url: str
    total_time_ms: float
    workers: list[WorkerResult] = field(default_factory=list)
    decomposition_time_ms: float = 0.0
    fusion_time_ms: float = 0.0
    errors: list[str] = field(default_factory=list)

    def to_dict(self) -> dict:
        return {
            "task_description": self.task_description,
            "num_workers": self.num_workers,
            "model": self.model,
            "base_url": self.base_url,
            "total_time_ms": round(self.total_time_ms, 1),
            "decomposition_time_ms": round(self.decomposition_time_ms, 1),
            "fusion_time_ms": round(self.fusion_time_ms, 1),
            "workers": [asdict(w) for w in self.workers],
            "errors": self.errors,
        }


# ---------------------------------------------------------------------------
# Task decomposition prompt — asks the model to split a task into subtasks
# ---------------------------------------------------------------------------

DECOMPOSE_PROMPT = """\
You are a task planner. Given the following coding task, decompose it into \
exactly {num_subtasks} independent subtasks that can be executed in parallel.

Each subtask must:
- Be self-contained (no dependencies on other subtasks' outputs)
- Have a clear, verifiable output
- Be completable by a single coding agent in one session

Return ONLY a JSON array (no markdown, no explanation):
[
  {{"id": "1", "title": "...", "description": "..."}},
  ...
]

Task: {task}
"""


# ---------------------------------------------------------------------------
# Worker prompt template — what each worker receives
# ---------------------------------------------------------------------------

WORKER_PROMPT_TEMPLATE = """\
You are a coding agent working on this subtask:

Title: {title}
Description: {description}

Context for the overall project:
{context}

Complete this subtask. Produce concrete code changes or a clear report \
of findings. Be specific — cite file paths and line numbers where possible.

If the subtask requires reading files, describe what you would read and why.
If it requires writing code, provide the actual code with file paths.

Be thorough but focused on this specific subtask only.\
"""


# ---------------------------------------------------------------------------
# Fusion prompt — merges worker results into a unified output
# ---------------------------------------------------------------------------

FUSION_PROMPT = """\
You are a synthesizer. You received results from {num_workers} parallel \
workers who each completed an independent subtask.

Merge their results into a single coherent output. Preserve all unique \
findings, resolve any conflicts, and produce a unified result.

Worker results:
{results}

Produce the merged output.\
"""


# ---------------------------------------------------------------------------
# API helpers
# ---------------------------------------------------------------------------

def create_client(base_url: str) -> OpenAI:
    """Create an OpenAI-compatible client pointing at a local model server."""
    return OpenAI(
        base_url=base_url,
        api_key="",  # local servers often don't require auth
    )


def call_chat(client: OpenAI, prompt: str, model: str, timeout: float) -> tuple[bool, str]:
    """Call the chat completions API. Returns (ok, response_text)."""
    try:
        resp = client.chat.completions.create(
            model=model,
            messages=[{"role": "user", "content": prompt}],
            temperature=0.3,
            max_tokens=4096,
            timeout=timeout,
        )
        if resp.choices:
            return True, resp.choices[0].message.content
        return False, "No choices returned"
    except Exception as e:
        return False, str(e)


# ---------------------------------------------------------------------------
# Memory measurement helpers
# ---------------------------------------------------------------------------

def get_rss_mb() -> float:
    """Get current process RSS in MB. Falls back to 0 if psutil unavailable."""
    if psutil is None:
        # Fallback: try /proc/self/status on Linux
        try:
            with open("/proc/self/status") as f:
                for line in f:
                    if line.startswith("VmRSS:"):
                        return int(line.split()[1]) / 1024.0
        except Exception:
            pass
        return 0.0

    proc = psutil.Process(os.getpid())
    mem_info = proc.memory_info()
    return mem_info.rss / (1024 * 1024)


# ---------------------------------------------------------------------------
# Task decomposition
# ---------------------------------------------------------------------------

def decompose_task(
    client: OpenAI, task: str, num_subtasks: int, model: str, timeout: float
) -> list[dict] | None:
    """Ask the model to decompose a task into independent subtasks."""
    prompt = DECOMPOSE_PROMPT.format(num_subtasks=num_subtasks, task=task)
    ok, response = call_chat(client, prompt, model, timeout)

    if not ok:
        return None

    # Extract JSON from response (handle markdown code blocks)
    text = response.strip()
    if "```" in text:
        # Extract JSON from code block
        start = text.find("```")
        end = text.rfind("```")
        if start != -1 and end != -1 and end > start:
            text = text[start + 3 : end].strip()

    try:
        subtasks = json.loads(text)
        if isinstance(subtasks, list) and len(subtasks) > 0:
            return subtasks
    except json.JSONDecodeError:
        pass

    print(f"  WARNING: Could not parse subtasks from model response.")
    return None


# ---------------------------------------------------------------------------
# Worker execution (runs in a thread)
# ---------------------------------------------------------------------------

def run_worker(
    worker_id: str,
    subtask: dict,
    context: str,
    client: OpenAI,
    model: str,
    timeout: float,
) -> WorkerResult:
    """Execute a single worker's subtask and measure its resources."""
    start = time.monotonic()
    rss_samples: list[float] = []

    try:
        # Sample memory periodically during execution
        sample_interval = 0.5  # seconds between RSS samples
        last_sample = start

        prompt = WORKER_PROMPT_TEMPLATE.format(
            title=subtask.get("title", ""),
            description=subtask.get("description", ""),
            context=context,
        )

        ok, output = call_chat(client, prompt, model, timeout)
        elapsed_ms = (time.monotonic() - start) * 1000

        # Final RSS sample
        rss_samples.append(get_rss_mb())

        if ok:
            return WorkerResult(
                worker_id=worker_id,
                subtask_index=int(subtask.get("id", 0)),
                status="success",
                elapsed_ms=elapsed_ms,
                rss_mb=max(rss_samples) if rss_samples else 0.0,
                output_length=len(output),
            )
        else:
            return WorkerResult(
                worker_id=worker_id,
                subtask_index=int(subtask.get("id", 0)),
                status="error",
                elapsed_ms=elapsed_ms,
                rss_mb=max(rss_samples) if rss_samples else 0.0,
                output_length=0,
                error=output,
            )

    except Exception as e:
        elapsed_ms = (time.monotonic() - start) * 1000
        return WorkerResult(
            worker_id=worker_id,
            subtask_index=int(subtask.get("id", 0)),
            status="error",
            elapsed_ms=elapsed_ms,
            rss_mb=get_rss_mb(),
            output_length=0,
            error=str(e),
        )


# ---------------------------------------------------------------------------
# Fusion step
# ---------------------------------------------------------------------------

def fuse_results(
    client: OpenAI, subtasks: list[dict], workers: list[WorkerResult], model: str, timeout: float
) -> tuple[str, float]:
    """Merge worker results into a unified output."""
    start = time.monotonic()

    # Build a summary of worker outputs
    summaries = []
    for w in workers:
        if w.status == "success":
            # Truncate long outputs for the fusion prompt
            output_preview = w.output[:2000] if len(w.output) > 2000 else w.output
            summaries.append(
                f"--- Subtask {w.subtask_index} (worker={w.worker_id}, "
                f"time={w.elapsed_ms:.0f}ms, rss={w.rss_mb:.1f}MB) ---\n{output_preview}"
            )
        else:
            summaries.append(
                f"--- Subtask {w.subtask_index} (worker={w.worker_id}, "
                f"FAILED: {w.error}) ---"
            )

    results_text = "\n\n".join(summaries)
    prompt = FUSION_PROMPT.format(
        num_workers=len(workers), results=results_text
    )

    ok, fused_output = call_chat(client, prompt, model, timeout)
    elapsed_ms = (time.monotonic() - start) * 1000

    if ok:
        return fused_output, elapsed_ms
    else:
        return f"FUSION ERROR: {fused_output}", elapsed_ms


# ---------------------------------------------------------------------------
# Single-agent baseline (for comparison)
# ---------------------------------------------------------------------------

def run_single_agent(
    client: OpenAI, task: str, model: str, timeout: float
) -> tuple[str, float]:
    """Run the task with a single agent (no decomposition)."""
    start = time.monotonic()
    ok, output = call_chat(client, task, model, timeout)
    elapsed_ms = (time.monotonic() - start) * 1000

    if ok:
        return output, elapsed_ms
    else:
        return f"ERROR: {output}", elapsed_ms


# ---------------------------------------------------------------------------
# Pretty printing
# ---------------------------------------------------------------------------

def print_header(title: str):
    width = 70
    print(f"\n{'=' * width}")
    print(f"  {title}")
    print(f"{'=' * width}")


def print_results(result: BenchmarkResult):
    """Print a human-readable summary of benchmark results."""
    print_header("BENCHMARK RESULTS")

    print(f"  Model:        {result.model}")
    print(f"  Base URL:     {result.base_url}")
    print(f"  Workers:      {result.num_workers}")
    print(f"  Task:         {result.task_description[:80]}...")
    print()

    # Timing breakdown
    print("  Timing:")
    print(f"    Decomposition: {result.decomposition_time_ms:.0f}ms")
    print(f"    Fusion:        {result.fusion_time_ms:.0f}ms")
    print(f"    Total:         {result.total_time_ms:.0f}ms")
    print()

    # Worker details
    print("  Workers:")
    for w in result.workers:
        status_icon = "✓" if w.status == "success" else "✗"
        print(
            f"    {status_icon} Worker-{w.worker_id}: "
            f"subtask={w.subtask_index}, "
            f"time={w.elapsed_ms:.0f}ms, "
            f"rss={w.rss_mb:.1f}MB, "
            f"output={w.output_length} chars"
        )
        if w.error:
            print(f"      ERROR: {w.error[:120]}")

    # Summary stats
    successful = [w for w in result.workers if w.status == "success"]
    failed = [w for w in result.workers if w.status != "success"]

    print()
    print("  Summary:")
    print(f"    Successful: {len(successful)}/{result.num_workers}")
    if failed:
        print(f"    Failed:     {len(failed)}")
        for f in failed:
            print(f"      - Worker-{f.worker_id}: {f.error[:80]}")

    # RAM analysis
    if successful:
        total_rss = sum(w.rss_mb for w in successful)
        avg_rss = total_rss / len(successful)
        max_rss = max(w.rss_mb for w in successful)
        print(f"\n  RAM Usage:")
        print(f"    Total (sum):     {total_rss:.1f} MB")
        print(f"    Average per:     {avg_rss:.1f} MB")
        print(f"    Peak single:     {max_rss:.1f} MB")

        # Check against 500MB budget for 10 workers
        if result.num_workers >= 10 and total_rss > 500:
            print(f"\n    ⚠ RAM budget exceeded! {result.num_workers} workers used "
                  f"{total_rss:.0f}MB (budget: 500MB)")
        else:
            print(f"\n    ✓ RAM within budget")

    # Errors
    if result.errors:
        print(f"\n  Errors ({len(result.errors)}):")
        for e in result.errors:
            print(f"    - {e}")


# ---------------------------------------------------------------------------
# Main benchmark runner
# ---------------------------------------------------------------------------

def run_benchmark(
    task: str,
    num_workers: int,
    model: str,
    base_url: str,
    timeout: float,
    task_file: Optional[str],
):
    """Run the full benchmark: decompose → parallel workers → fuse."""

    # Load task from file if provided
    if task_file:
        task_path = Path(task_file)
        if not task_path.exists():
            print(f"ERROR: Task file not found: {task_file}")
            sys.exit(1)
        task = task_path.read_text().strip()

    print_header("LOCAL SWARM BENCHMARK")
    print(f"  Model:        {model}")
    print(f"  Base URL:     {base_url}")
    print(f"  Workers:      {num_workers}")
    print(f"  Timeout:      {timeout}s")
    print(f"  Task:         {task[:100]}...")

    overall_start = time.monotonic()
    client = create_client(base_url)

    # --- Phase 1: Task decomposition ---
    print_header("PHASE 1: Task Decomposition")

    subtasks = decompose_task(client, task, num_workers, model, timeout)
    if subtasks is None:
        print("  ✗ FAILED: Could not decompose task.")
        result = BenchmarkResult(
            task_description=task,
            num_workers=num_workers,
            model=model,
            base_url=base_url,
            total_time_ms=(time.monotonic() - overall_start) * 1000,
            errors=["Task decomposition failed"],
        )
        print_results(result)
        return result

    print(f"  ✓ Decomposed into {len(subtasks)} subtasks:")
    for st in subtasks:
        print(f"    [{st.get('id', '?')}] {st.get('title', 'unnamed')}")

    # --- Phase 2: Parallel worker execution ---
    print_header("PHASE 2: Parallel Worker Execution")

    # Capture baseline memory before spawning workers
    baseline_rss = get_rss_mb()
    print(f"  Baseline RSS: {baseline_rss:.1f} MB")

    workers: list[WorkerResult] = []
    with ThreadPoolExecutor(max_workers=num_workers) as pool:
        futures = {}
        for i, subtask in enumerate(subtasks):
            worker_id = f"{i:03d}"
            future = pool.submit(
                run_worker, worker_id, subtask, task, client, model, timeout
            )
            futures[future] = worker_id

        for future in as_completed(futures):
            try:
                w = future.result()
                workers.append(w)
                status_icon = "✓" if w.status == "success" else "✗"
                print(
                    f"  {status_icon} Worker-{w.worker_id}: "
                    f"time={w.elapsed_ms:.0f}ms, rss={w.rss_mb:.1f}MB"
                )
            except Exception as e:
                print(f"  ✗ Worker {futures[future]} raised: {e}")

    # Sort workers by subtask index for consistent output
    workers.sort(key=lambda w: w.subtask_index)

    # --- Phase 3: Fusion ---
    print_header("PHASE 3: Result Fusion")

    fused_output, fusion_ms = fuse_results(
        client, subtasks, workers, model, timeout
    )
    print(f"  Fusion time: {fusion_ms:.0f}ms")

    # --- Compute totals ---
    total_time_ms = (time.monotonic() - overall_start) * 1000
    result = BenchmarkResult(
        task_description=task,
        num_workers=num_workers,
        model=model,
        base_url=base_url,
        total_time_ms=total_time_ms,
        decomposition_time_ms=(time.monotonic() - overall_start) * 1000,
        fusion_time_ms=fusion_ms,
        workers=workers,
    )

    # Print results
    print_results(result)

    # --- Single-agent baseline for comparison ---
    print_header("BASELINE: Single Agent (no decomposition)")

    single_output, single_time_ms = run_single_agent(
        client, task, model, timeout
    )
    print(f"  Single agent time: {single_time_ms:.0f}ms")

    # Compare
    swarm_workers_time = max(w.elapsed_ms for w in workers if w.status == "success") if any(
        w.status == "success" for w in workers
    ) else 0

    if swarm_workers_time > 0 and single_time_ms > 0:
        speedup = single_time_ms / swarm_workers_time
        print(f"\n  Comparison:")
        print(f"    Single agent: {single_time_ms:.0f}ms")
        print(f"    Swarm (parallel): ~{swarm_workers_time:.0f}ms (wall time, longest worker)")
        if speedup > 1:
            print(f"    Speedup: {speedup:.2f}x (single is faster)")
        elif speedup < 1:
            print(f"    Slowdown: {1/speedup:.2f}x (swarm is faster)")
        else:
            print(f"    Equivalent timing")

    # --- Output JSON ---
    print_header("JSON OUTPUT")
    json_output = result.to_dict()
    # Add baseline for comparison
    json_output["baseline_single_agent"] = {
        "time_ms": round(single_time_ms, 1),
        "output_length": len(single_output),
    }
    print(json.dumps(json_output, indent=2))

    return result


# ---------------------------------------------------------------------------
# CLI entry point
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(
        description="Benchmark swarm task decomposition with local models",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""\
Examples:
  # Default (Ollama on localhost)
  python scripts/benchmark_local_swarm.py

  # Custom model and worker count
  python scripts/benchmark_local_swarm.py --workers 10 --model qwen2.5:7b

  # OMLX endpoint
  python scripts/benchmark_local_swarm.py --base-url http://localhost:18000/v1

  # Custom task from file
  python scripts/benchmark_local_swarm.py --task-file my_task.txt

  # Quick smoke test (1 worker, short timeout)
  python scripts/benchmark_local_swarm.py --workers 3 --timeout 60
""",
    )

    parser.add_argument(
        "--workers", "-w",
        type=int,
        default=5,
        help="Number of parallel workers (default: 5)",
    )
    parser.add_argument(
        "--model", "-m",
        type=str,
        default=os.environ.get("BENCHMARK_MODEL", "qwen2.5:7b"),
        help="Model name for local server (default: qwen2.5:7b)",
    )
    parser.add_argument(
        "--base-url", "-u",
        type=str,
        default=os.environ.get("BENCHMARK_BASE_URL", "http://localhost:11434/v1"),
        help="OpenAI-compatible API base URL (default: http://localhost:11434/v1)",
    )
    parser.add_argument(
        "--timeout", "-t",
        type=float,
        default=300.0,
        help="Per-request timeout in seconds (default: 300)",
    )
    parser.add_argument(
        "--task", "-T",
        type=str,
        default="Refactor the authentication module to support OAuth2 PKCE flow. Identify all files that need changes, propose the modifications, and write the code for the auth provider abstraction layer.",
        help="Task description (default: a sample refactoring task)",
    )
    parser.add_argument(
        "--task-file", "-f",
        type=str,
        default=None,
        help="Path to a file containing the task description",
    )

    args = parser.parse_args()

    if args.workers < 1:
        print("ERROR: --workers must be >= 1")
        sys.exit(1)

    result = run_benchmark(
        task=args.task,
        num_workers=args.workers,
        model=args.model,
        base_url=args.base_url,
        timeout=args.timeout,
        task_file=args.task_file,
    )

    # Exit code: 0 if all workers succeeded, 1 otherwise
    failed = [w for w in result.workers if w.status != "success"]
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
