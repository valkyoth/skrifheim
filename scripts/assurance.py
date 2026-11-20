#!/usr/bin/env python3
"""Bounded local fuzz campaigns and stable deterministic corpus checks."""
import argparse
import hashlib
import os
from pathlib import Path
import re
import signal
import subprocess

ROOT = Path(__file__).resolve().parent.parent
TARGETS = {
    "wal_header": 121, "wal_replay": 642, "segment_metadata": 513,
    "segment_layout": 2048, "aead_envelope": 65596, "transcript": 4096,
}
NIGHTLY = "nightly-2026-11-20"


def selected(name):
    if name == "all":
        return list(TARGETS)
    if name not in TARGETS:
        raise ValueError("unknown assurance target")
    return [name]


def require_cases(output, names):
    rows = re.findall(r"^([a-z_]+): ([0-9]+) cases$", output, re.M)
    if len(rows) != len(names) or {name for name, _ in rows} != set(names):
        raise ValueError("missing, duplicate or unexpected target result")
    if any(int(count) <= 0 for _, count in rows):
        raise ValueError("zero cases is incomplete")


def require_fuzz(output, runs):
    finished = re.findall(r"^Done ([0-9]+) runs in [0-9]+ second\(s\)$", output, re.M)
    coverage = re.findall(r"cov: ([0-9]+)", output)
    if len(finished) != 1 or int(finished[0]) < runs:
        raise ValueError("fuzz run did not complete its case budget")
    if not coverage or max(map(int, coverage)) == 0:
        raise ValueError("missing coverage-guided execution evidence")


def run(command, timeout=900):
    with subprocess.Popen(command, cwd=ROOT, text=True, stdout=subprocess.PIPE,
                          stderr=subprocess.STDOUT, start_new_session=os.name == "posix") as process:
        try:
            output, _ = process.communicate(timeout=timeout)
        except (subprocess.TimeoutExpired, KeyboardInterrupt):
            if os.name == "posix":
                os.killpg(process.pid, signal.SIGKILL)
            else:
                process.kill()
            process.communicate()
            raise
        if process.returncode:
            print(output, flush=True)
            raise RuntimeError(f"command failed ({process.returncode}): {command}")
        return output


def campaign(names, runs):
    directory = ROOT / "target" / "assurance"
    directory.mkdir(parents=True, exist_ok=True)
    lock = ROOT / "crates/skrifheim-fuzz/Cargo.lock"
    before = hashlib.sha256(lock.read_bytes()).digest()
    for name in names:
        corpus = directory / "corpus" / name
        run(["cargo", "run", "--locked", "-q", "-p", "skrifheim-assurance", "--",
             "seeds", name, str(corpus)])
        print(f"Fuzzing {name}: {runs} runs, seed 184, ASan", flush=True)
        output = run(["cargo", f"+{NIGHTLY}", "fuzz", "run", name, str(corpus),
                      "--fuzz-dir", "crates/skrifheim-fuzz", "--target-dir", "target/fuzz", "--",
                      f"-runs={runs}", "-seed=184", f"-max_len={TARGETS[name]}",
                      "-timeout=5", "-rss_limit_mb=512"])
        (directory / f"{name}.log").write_text(output)
        require_fuzz(output, runs)
        print(f"{name}: completed {runs} or more coverage-guided runs", flush=True)
    if hashlib.sha256(lock.read_bytes()).digest() != before:
        raise ValueError("fuzz dependency lock changed during execution")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=["smoke", "fuzz"])
    parser.add_argument("target", choices=["all", *TARGETS], default="all", nargs="?")
    parser.add_argument("--runs", type=int, default=20000)
    args = parser.parse_args()
    if args.runs < 1000 or args.runs > 10000000:
        parser.error("runs must be in 1000..10000000")
    names = selected(args.target)
    if args.mode == "smoke":
        output = run(["cargo", "run", "--locked", "-q", "-p", "skrifheim-assurance",
                      "--", "smoke", args.target])
        require_cases(output, names)
        print(output, end="")
    else:
        campaign(names, args.runs)


if __name__ == "__main__":
    main()
