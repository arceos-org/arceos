#!/usr/bin/env python3
"""Check real axalloc OOM and axlibc's C allocation boundary in isolated processes."""
import argparse
import json
from pathlib import Path
import resource
import signal
import subprocess

ROOT = Path(__file__).resolve().parent
MODES = ["fallible", "small", "trait-oom", "malloc-oom", "size-max",
         "layout-limit", "max-valid", "rust-vec-oom"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--release", action="store_true")
    args = parser.parse_args()
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    profile = "release" if args.release else "debug"
    command = ["cargo", "build", "--locked", "--target-dir", str(ROOT / "target")]
    if args.release:
        command.append("--release")
    subprocess.run(command, cwd=ROOT, check=True)
    rows = []
    for mode in MODES:
        proc = subprocess.run([str(ROOT / "target" / profile / "arceos-oom-probe"), mode],
                              capture_output=True, text=True, timeout=15)
        if mode == "rust-vec-oom":
            matched = (proc.returncode == -signal.SIGABRT and
                       "memory allocation of 1073741832 bytes failed" in proc.stderr)
        else:
            matched = proc.returncode == 0 and "PASS:" in proc.stdout
        rows.append({"mode": mode, "returncode": proc.returncode,
                     "passed": matched, "stdout": proc.stdout, "stderr": proc.stderr})
        print(f"{'PASS' if matched else 'FAIL'} {mode}: exit {proc.returncode}")
        if not matched:
            print(proc.stdout + proc.stderr)
    result = {"rustc": subprocess.check_output(["rustc", "--version"], text=True).strip(),
              "profile": profile, "results": rows}
    (ROOT / "target" / f"oom-{profile}.json").write_text(json.dumps(result, indent=2) + "\n")
    if not all(row["passed"] for row in rows):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
