#!/usr/bin/env bash
# check-workflow-permissions.sh
#
# Regression check for issue #7 (P0.1): a GitHub Actions job that holds
# `contents: write` (declared on the job, or inherited from workflow-level
# `permissions`) must not install dependencies, build, run migrations, or
# execute repository scripts. Generation and publication are separate trust
# boundaries; see docs/architecture/evidence/VG-01-publication-procedure.md.
#
# Usage: scripts/check-workflow-permissions.sh [workflow-dir]
#   workflow-dir defaults to .github/workflows (relative to the repo root).
#
# Parser: a deliberately small indentation-based reader (python3 stdlib only).
# It understands `permissions:` blocks, `permissions: write-all`, inline
# `{ contents: write }`, `steps:` and `run:` scalars. Anything it cannot
# classify as read-only is treated conservatively.

set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
dir="${1:-$root/.github/workflows}"

if [[ ! -d "$dir" ]]; then
  echo "check-workflow-permissions: directory not found: $dir" >&2
  exit 2
fi

shopt -s nullglob
files=("$dir"/*.yml "$dir"/*.yaml)
if [[ ${#files[@]} -eq 0 ]]; then
  echo "check-workflow-permissions: no workflow files in $dir" >&2
  exit 2
fi

python3 -I - "${files[@]}" <<'PY'
import re
import sys

# Commands a write-capable job must never run.
RULES = [
    (re.compile(r"\b(bun|npm|pnpm|yarn)\s+(install|i|ci|add|run|test|build|exec|x)\b"),
     "dependency install or package-manager build/test"),
    (re.compile(r"\b(npx|bunx)\b"), "package executor"),
    (re.compile(r"\bcargo\s+(build|test|install|check|clippy|run|fetch|bench|doc)\b"),
     "cargo build/test/install"),
    (re.compile(r"\b(rustc|rustup)\b"), "rust toolchain/compiler"),
    (re.compile(r"\bpip3?\s+install\b|\bpython3?\s+-m\s+pip\b"), "pip install"),
    (re.compile(r"migrat", re.I), "database migration"),
    (re.compile(
        r"(?:^|[;&|(]\s*)(?:(?:bash|sh|zsh|source|python3?|node|bun|deno)\s+)?"
        r"(?:\./|tools/|scripts/|\.legacy/)"),
     "repository script execution"),
]
LOCAL_ACTION = re.compile(r"^\s*-?\s*uses:\s*['\"]?\./")


def indent(line):
    return len(line) - len(line.lstrip(" "))


def writes_contents(block_lines, header_value):
    """True if a permissions declaration grants contents: write."""
    if header_value:
        v = header_value.strip()
        if v == "write-all":
            return True
        if re.search(r"\bcontents\s*:\s*write\b", v):
            return True
        return False
    for ln in block_lines:
        s = ln.strip()
        if re.match(r"contents\s*:\s*write\b", s):
            return True
    return False


def find_permissions(lines, key_indent):
    """Return (found, writes, end_index) for a `permissions:` key at key_indent."""
    for i, ln in enumerate(lines):
        if ln.strip().startswith("#") or not ln.strip():
            continue
        if indent(ln) == key_indent:
            m = re.match(r"\s*permissions\s*:\s*(.*?)\s*(#.*)?$", ln)
            if m:
                val = m.group(1)
                body = []
                j = i + 1
                while j < len(lines) and (not lines[j].strip() or indent(lines[j]) > key_indent):
                    body.append(lines[j])
                    j += 1
                return True, writes_contents(body, val), j
    return False, False, 0


def run_lines(job_lines):
    """Yield (lineno_in_job, text) for every shell line under `run:` keys."""
    i = 0
    n = len(job_lines)
    while i < n:
        ln = job_lines[i]
        m = re.match(r"^(\s*)(?:-\s+)?run\s*:\s*(.*)$", ln)
        if not m:
            i += 1
            continue
        key_indent = len(m.group(1)) + (2 if re.match(r"^\s*-\s+run", ln) else 0)
        rest = m.group(2).strip()
        if rest and rest[0] not in "|>":
            yield i, rest
            i += 1
            continue
        i += 1
        continued = False
        while i < n and (not job_lines[i].strip() or indent(job_lines[i]) > key_indent):
            text = job_lines[i].strip()
            if text and not text.startswith("#"):
                # Continuation lines are arguments, not new commands.
                if not continued:
                    yield i, text
                continued = text.endswith("\\")
            i += 1


def split_jobs(lines):
    """Return list of (name, start_index, job_lines) for jobs under top-level `jobs:`."""
    start = None
    for i, ln in enumerate(lines):
        if re.match(r"^jobs\s*:\s*$", ln):
            start = i + 1
            break
    if start is None:
        return []
    jobs = []
    cur = None
    for i in range(start, len(lines)):
        ln = lines[i]
        if not ln.strip() or ln.strip().startswith("#"):
            if cur:
                cur[2].append(ln)
            continue
        if indent(ln) == 0:
            break
        m = re.match(r"^(\s+)([A-Za-z0-9_-]+)\s*:\s*$", ln)
        if m and len(m.group(1)) == 2:
            cur = [m.group(2), i, []]
            jobs.append(cur)
            continue
        if cur:
            cur[2].append(ln)
    return jobs


def main(paths):
    failures = []
    checked_write_jobs = 0
    for path in sorted(paths):
        with open(path, encoding="utf-8") as fh:
            lines = fh.read().splitlines()
        top_found, top_writes, _ = find_permissions(
            [ln for ln in lines if ln.strip()], 0)
        # find_permissions on filtered lines only looks at indent-0 keys, which
        # is exactly the workflow-level declaration.
        for name, start, jlines in split_jobs(lines):
            found, writes, _ = find_permissions(jlines, 4)
            effective_write = writes if found else top_writes
            if not effective_write:
                continue
            checked_write_jobs += 1
            for off, text in run_lines(jlines):
                for rx, why in RULES:
                    if rx.search(text):
                        failures.append(
                            f"{path}:{start + 2 + off}: job '{name}' has contents: write "
                            f"but runs {why}: {text}")
                        break
            for off, ln in enumerate(jlines):
                if LOCAL_ACTION.match(ln):
                    failures.append(
                        f"{path}:{start + 2 + off}: job '{name}' has contents: write "
                        f"but uses a local repository action: {ln.strip()}")
    if failures:
        print("check-workflow-permissions: FAIL", file=sys.stderr)
        for f in failures:
            print(f"  {f}", file=sys.stderr)
        return 1
    print(f"check-workflow-permissions: OK "
          f"({len(paths)} workflow file(s), {checked_write_jobs} write-capable job(s) inspected)")
    return 0


sys.exit(main(sys.argv[1:]))
PY
