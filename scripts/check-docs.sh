#!/usr/bin/env bash
#
# check-docs.sh — the executable documentation gate for AWBMS.
#
# The architecture validation states (§2.1) that registries, contracts and
# executable checks outrank prose. This script is that rule applied to the
# repository's own documents. It fails the build when a document drifts from
# what it asserts.
#
# Checks:
#   1  VERSION is a bare X.Y.Z
#   2  CHANGELOG.md has a released section for VERSION
#   3  The architecture validation header states the same version
#   4  Every AD-/OD-/VG-/C-/AS- identifier referenced has a register row
#   5  Every internal (#anchor) link resolves to a heading in the same file
#   6  No duplicate heading slugs (which would make anchors ambiguous)
#   7  Repository independence (VG-15): no AWCMS submodule or build dependency
#   8  Every changeset is well-formed
#   9  Scripts are executable
#   10 Gate register (docs/architecture/gates.json) agrees with the gates
#      document and the architecture validation; PASS gates carry evidence,
#      a commit and a run; no gate with an existing check is "unautomated"
#
# Usage: scripts/check-docs.sh [--quiet]

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

ARCH_DOC="docs/architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md"
QUIET=0
[ "${1:-}" = "--quiet" ] && QUIET=1

FAILURES=0
CHECKS=0

pass() { CHECKS=$((CHECKS + 1)); [ "$QUIET" -eq 1 ] || printf '  ok    %s\n' "$1"; }
fail() { CHECKS=$((CHECKS + 1)); FAILURES=$((FAILURES + 1)); printf '  FAIL  %s\n' "$1" >&2; }
note() { [ "$QUIET" -eq 1 ] || printf '        %s\n' "$1"; }
group() { [ "$QUIET" -eq 1 ] || printf '\n%s\n' "$1"; }

# Emit the GitHub-flavoured anchor slug for each ATX heading on stdin's file,
# ignoring headings inside fenced code blocks.
heading_slugs() {
  awk '
    /^```/ { fence = !fence; next }
    fence { next }
    /^#{1,6}[ \t]/ {
      line = $0
      sub(/^#{1,6}[ \t]+/, "", line)
      sub(/[ \t]+#*[ \t]*$/, "", line)
      print line
    }
  ' "$1" | LC_ALL=C tr '[:upper:]' '[:lower:]' \
         | LC_ALL=C sed -e 's/[^a-z0-9 _-]//g' -e 's/ /-/g'
}

# Emit every markdown link target in the file, one per line, excluding code
# fences and external URLs.
link_targets() {
  awk '
    /^```/ { fence = !fence; next }
    fence { next }
    { print }
  ' "$1" | grep -o '](\([^)"[:space:]]*\))' \
         | sed -e 's/^](//' -e 's/)$//' \
         | grep -v '^[a-z][a-z0-9+.-]*://' \
         | grep -v '^mailto:'
}

# --------------------------------------------------------------- 1..3 version
group "Version consistency"

VERSION=""
if [ ! -f VERSION ]; then
  fail "VERSION file is missing"
else
  VERSION="$(tr -d '[:space:]' < VERSION)"
  if printf '%s' "$VERSION" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    pass "VERSION is a bare X.Y.Z ($VERSION)"
  else
    fail "VERSION is not a bare X.Y.Z: '$VERSION'"
  fi
fi

if [ ! -f CHANGELOG.md ]; then
  fail "CHANGELOG.md is missing"
elif [ -z "$VERSION" ]; then
  fail "cannot check CHANGELOG.md without a valid VERSION"
elif grep -q "^## \[$VERSION\]" CHANGELOG.md; then
  pass "CHANGELOG.md has a released section for $VERSION"
else
  fail "CHANGELOG.md has no '## [$VERSION]' section"
fi

if [ ! -f "$ARCH_DOC" ]; then
  fail "$ARCH_DOC is missing"
elif [ -z "$VERSION" ]; then
  fail "cannot check the architecture header without a valid VERSION"
elif grep -qF "| Specification version | v$VERSION |" "$ARCH_DOC"; then
  pass "architecture validation header states v$VERSION"
else
  fail "architecture validation header does not state v$VERSION"
  note "expected the row: | Specification version | v$VERSION |"
fi

# ------------------------------------------------------------ 4 register IDs
group "Register integrity (every referenced identifier has a register row)"

if [ -f "$ARCH_DOC" ]; then
  IDS="$(grep -o '\b\(AD\|OD\|VG\|AS\|C\)-[0-9][0-9]\b' "$ARCH_DOC" | sort -u)"
  MISSING=""
  COUNT=0
  for id in $IDS; do
    COUNT=$((COUNT + 1))
    # A register row is a table row whose first cell is the backticked ID.
    if ! grep -q "^| \`$id\` |" "$ARCH_DOC"; then
      MISSING="$MISSING $id"
    fi
  done
  if [ -z "$MISSING" ]; then
    pass "all $COUNT referenced identifiers have a register row"
  else
    fail "identifiers referenced but never registered:$MISSING"
  fi

  # And the reverse: a register row nobody references is dead weight.
  ORPHANS=""
  while IFS= read -r row_id; do
    [ -z "$row_id" ] && continue
    refs="$(grep -c "\`$row_id\`" "$ARCH_DOC")"
    [ "$refs" -le 1 ] && ORPHANS="$ORPHANS $row_id"
  done <<EOF
$(grep -o '^| `\(AD\|OD\|VG\|AS\|C\)-[0-9][0-9]` |' "$ARCH_DOC" | sed -e 's/^| `//' -e 's/` |$//' | sort -u)
EOF
  if [ -z "$ORPHANS" ]; then
    pass "no register row is referenced only by itself"
  else
    fail "registered but never referenced elsewhere:$ORPHANS"
  fi
fi

# ---------------------------------------------------------------- 5..6 links
group "Internal link integrity"

# Frozen legacy fixtures under contracts/legacy/ are verbatim copies of external
# artifacts, checksum-pinned by their manifests (VG-01). Their links point into
# the AWCMS tree and do not resolve here — correctly so. Rewriting them to
# satisfy a link checker would invalidate the SHA-256 provenance that makes them
# evidence, so they are excluded from documentation checks by design.
DOCS="$(find . -name '*.md' -not -path './.git/*' -not -path './contracts/legacy/*' \
        -not -path './target/*' | sort)"

SLUG_CACHE_DIR="$(mktemp -d)"
trap 'rm -rf "$SLUG_CACHE_DIR"' EXIT

slugs_for() {
  # Cache heading slugs per file; $1 is a repo-relative path like ./docs/x.md
  local key cache
  key="$(printf '%s' "$1" | tr '/.' '__')"
  cache="$SLUG_CACHE_DIR/$key"
  [ -f "$cache" ] || heading_slugs "$1" > "$cache" 2>/dev/null
  cat "$cache"
}

BROKEN_TOTAL=0
LINKED_SLUGS="$SLUG_CACHE_DIR/.linked"
: > "$LINKED_SLUGS"

for doc in $DOCS; do
  dir="$(dirname "$doc")"
  broken=""

  for target in $(link_targets "$doc" | sort -u); do
    case "$target" in
      \#*)
        # Same-file anchor.
        anchor="${target#\#}"
        printf '%s\t%s\n' "$doc" "$anchor" >> "$LINKED_SLUGS"
        slugs_for "$doc" | grep -qx -- "$anchor" || broken="$broken $target"
        ;;
      *\#*)
        # Cross-file anchor: path#anchor
        path="${target%%\#*}"
        anchor="${target#*\#}"
        resolved="$(realpath -m --relative-to=. "$dir/$path" 2>/dev/null)"
        if [ ! -e "$resolved" ]; then
          broken="$broken $target(missing-file)"
        else
          case "$resolved" in
            *.md)
              printf '%s\t%s\n' "./$resolved" "$anchor" >> "$LINKED_SLUGS"
              slugs_for "./$resolved" | grep -qx -- "$anchor" \
                || broken="$broken $target(missing-anchor)"
              ;;
          esac
        fi
        ;;
      *)
        # Plain relative path.
        resolved="$(realpath -m --relative-to=. "$dir/$target" 2>/dev/null)"
        [ -e "$resolved" ] || broken="$broken $target(missing-file)"
        ;;
    esac
  done

  if [ -n "$broken" ]; then
    BROKEN_TOTAL=$((BROKEN_TOTAL + 1))
    fail "$doc: unresolved links:$broken"
  fi
done
[ "$BROKEN_TOTAL" -eq 0 ] && pass "every internal link and anchor resolves"

# A duplicate heading slug only matters if something actually links to it —
# repeated '### Added' headings across CHANGELOG releases are correct.
AMBIGUOUS=0
for doc in $DOCS; do
  dupes="$(slugs_for "$doc" | sed '/^$/d' | sort | uniq -d)"
  [ -z "$dupes" ] && continue
  hit=""
  for d in $dupes; do
    grep -qF "$(printf '%s\t%s' "$doc" "$d")" "$LINKED_SLUGS" 2>/dev/null && hit="$hit $d"
  done
  if [ -n "$hit" ]; then
    AMBIGUOUS=$((AMBIGUOUS + 1))
    fail "$doc: linked-to heading slugs are duplicated, so the anchor is ambiguous:$hit"
  fi
done
[ "$AMBIGUOUS" -eq 0 ] && pass "no linked-to heading slug is ambiguous"

# ------------------------------------------------------- 7 VG-15 independence
group "Repository independence (VG-15)"

if [ -f .gitmodules ]; then
  fail ".gitmodules exists — AWBMS must not submodule AWCMS (§3.2)"
else
  pass "no git submodules"
fi

COUPLED=0
for manifest in $(find . -name Cargo.toml -not -path './.git/*' -not -path './target/*' 2>/dev/null); do
  if grep -Eq '(path|git)[[:space:]]*=[[:space:]]*"[^"]*awcms' "$manifest"; then
    fail "$manifest declares a path/git dependency on AWCMS (§3.2)"
    COUPLED=1
  fi
done
[ "$COUPLED" -eq 0 ] && pass "no Cargo manifest depends on AWCMS"

# ------------------------------------------------------------- 8 changesets
group "Changeset well-formedness"

BAD=0
SEEN=0
for f in .changeset/*.md; do
  [ -e "$f" ] || continue
  case "$(basename "$f")" in README.md) continue ;; esac
  SEEN=$((SEEN + 1))
  head -n1 "$f" | grep -qx -- '---' || { fail "$f: does not start with '---' frontmatter"; BAD=1; continue; }
  for key in bump type component; do
    grep -qE "^$key:[[:space:]]*[a-z-]+[[:space:]]*$" "$f" || { fail "$f: missing or malformed '$key'"; BAD=1; }
  done
  grep -qE '^bump:[[:space:]]*(major|minor|patch)[[:space:]]*$' "$f" || { fail "$f: 'bump' must be major|minor|patch"; BAD=1; }
  grep -qE '^type:[[:space:]]*(added|changed|deprecated|removed|fixed|security)[[:space:]]*$' "$f" || { fail "$f: invalid 'type'"; BAD=1; }
done
if [ "$BAD" -eq 0 ]; then
  pass "$SEEN pending changeset(s), all well-formed"
fi

# --------------------------------------------------------------- 9 scripts
group "Script hygiene"

NOTEXEC=""
for s in scripts/*.sh; do
  [ -e "$s" ] || continue
  [ -x "$s" ] || NOTEXEC="$NOTEXEC $s"
done
if [ -z "$NOTEXEC" ]; then
  pass "all scripts are executable"
else
  fail "not executable (run chmod +x):$NOTEXEC"
fi

if command -v shellcheck >/dev/null 2>&1; then
  if shellcheck -S error scripts/*.sh >/dev/null 2>&1; then
    pass "shellcheck (severity=error) clean"
  else
    fail "shellcheck reported errors — run: shellcheck -S error scripts/*.sh"
  fi
else
  note "shellcheck not installed; skipped (optional)"
fi

# ------------------------------------------------------- 10 gate register
group "Gate register (docs/architecture/gates.json)"

GATES_JSON="docs/architecture/gates.json"
GATES_DOC="docs/architecture/AWBMS-STAGE-1-GATES.md"

# The register is the machine-readable source of gate status; the Markdown
# table, the architecture validation and the evidence are checked against it.
# Python 3 (stdlib only) parses the JSON; it prints one "ok"/"FAIL" line per
# finding, which is turned into pass/fail below.
if [ ! -f "$GATES_JSON" ]; then
  fail "$GATES_JSON is missing"
elif [ ! -f "$GATES_DOC" ]; then
  fail "$GATES_DOC is missing"
elif ! command -v python3 >/dev/null 2>&1; then
  fail "python3 is required to validate $GATES_JSON"
else
  SHALLOW="$(git rev-parse --is-shallow-repository 2>/dev/null || echo true)"
  GATES_DONE=0
  while IFS= read -r line; do
    case "$line" in
      "ok "*)   pass "${line#ok }" ;;
      "FAIL "*) fail "${line#FAIL }" ;;
      "__GATES_DONE__") GATES_DONE=1 ;;
      *)        note "$line" ;;
    esac
  done <<EOF
$(python3 -I - "$GATES_JSON" "$GATES_DOC" "$ARCH_DOC" "$SHALLOW" <<'PY'
import json, os, re, subprocess, sys

reg_path, doc_path, arch_path, shallow = sys.argv[1:5]
out = []
ok = lambda m: out.append("ok " + m)
bad = lambda m: out.append("FAIL " + m)

try:
    with open(reg_path, encoding="utf-8") as fh:
        reg = json.load(fh)
    gates = reg["gates"]
except (OSError, ValueError, KeyError) as exc:
    print("FAIL %s is not a valid register: %s" % (reg_path, exc))
    sys.exit(0)

STATUSES = {"PASS", "PARTIAL", "OPEN"}
HEX40 = re.compile(r"^[0-9a-f]{40}$")
ids = [g.get("id") for g in gates]
problems = []
if len(set(ids)) != len(ids):
    problems.append("duplicate gate ids")
for g in gates:
    gid = g.get("id", "?")
    for key in ("id", "title", "status", "evidence", "check", "established"):
        if key not in g:
            problems.append("%s lacks '%s'" % (gid, key))
    if g.get("status") not in STATUSES:
        problems.append("%s has status %r, expected one of %s" % (gid, g.get("status"), sorted(STATUSES)))
if problems:
    bad("register schema: " + "; ".join(problems))
    print("\n".join(out))
    sys.exit(0)
ok("register has %d well-formed gates" % len(gates))

# (a) The Markdown status table must equal the register exactly.
rows = []
with open(doc_path, encoding="utf-8") as fh:
    for line in fh:
        if line.startswith("| `VG-"):
            cells = [c.strip() for c in line.strip().strip("|").split("|")]
            rows.append((cells[0].strip("`"), cells[1], cells[2].replace("*", "").strip()))
want = [(g["id"], g["title"], g["status"]) for g in gates]
if rows == want:
    ok("gate table in %s matches the register exactly" % os.path.basename(doc_path))
else:
    diffs = []
    table = {r[0]: r for r in rows}
    for w in want:
        r = table.get(w[0])
        if r is None:
            diffs.append("%s missing from table" % w[0])
        elif r != w:
            diffs.append("%s table=%r register=%r" % (w[0], r[1:], w[1:]))
    for r in rows:
        if r[0] not in ids:
            diffs.append("%s in table but not in register" % r[0])
    if not diffs:
        diffs.append("row order differs from the register")
    bad("gate table differs from register: " + "; ".join(diffs))

# (b) A PASS gate needs existing evidence, a recorded commit and a run id.
for g in gates:
    gid, why = g["id"], []
    for path in g["evidence"]:
        if not os.path.exists(path):
            why.append("evidence path missing: " + path)
    if g["status"] == "PASS":
        if not g["evidence"]:
            why.append("no evidence paths")
        est = g["established"] or {}
        commit, run = est.get("commit", ""), est.get("run")
        if not HEX40.match(str(commit)):
            why.append("no valid 40-hex commit recorded")
        elif shallow != "true":
            r = subprocess.run(["git", "cat-file", "-e", commit + "^{commit}"],
                               stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            if r.returncode != 0:
                why.append("recorded commit %s is not in this repository" % commit[:12])
        if not isinstance(run, int) or isinstance(run, bool) or run <= 0:
            why.append("no workflow run id recorded")
    if why:
        bad("%s (%s): %s" % (gid, g["status"], "; ".join(why)))
    else:
        ok("%s (%s): evidence present%s" % (gid, g["status"],
            ", commit/run recorded" if g["status"] == "PASS" else ""))

# The registered executable check must exist, and no document may call a gate
# with an existing check unautomated.
automated = set()
for g in gates:
    chk = g["check"]
    if chk is None:
        continue
    if not isinstance(chk, dict):
        bad("%s: check must be null or an object with a kind, got %r" % (g["id"], chk))
        continue
    gid, kind = g["id"], chk.get("kind")
    if kind == "script":
        found = os.path.isfile(chk.get("path", ""))
    elif kind == "workflow":
        found = False
        if os.path.isfile(chk.get("file", "")):
            with open(chk["file"], encoding="utf-8") as fh:
                found = ("name: " + chk.get("job", "\0")) in fh.read()
    else:
        found = False
    if found:
        automated.add(gid)
        ok("%s: registered executable check exists" % gid)
    else:
        bad("%s: registered executable check not found: %r" % (gid, chk))

# (c) Prose must not call a gate unautomated while its check exists.
stale = []
for path in (doc_path, arch_path):
    if not os.path.isfile(path):
        continue
    with open(path, encoding="utf-8") as fh:
        for n, line in enumerate(fh, 1):
            if re.search(r"unautomated", line, re.I):
                for gid in re.findall(r"VG-\d\d", line):
                    if gid in automated:
                        stale.append("%s:%d (%s)" % (path, n, gid))
if stale:
    bad("described as unautomated although its executable check exists: " + ", ".join(stale))
else:
    ok("no gate with an existing executable check is described as unautomated")

# The architecture validation's Appendix E status must agree with the register.
if os.path.isfile(arch_path):
    arch = {}
    with open(arch_path, encoding="utf-8") as fh:
        for line in fh:
            m = re.match(r"^\| `(VG-\d\d)` \|", line)
            if m:
                cells = [c.strip() for c in line.strip().strip("|").split("|")]
                word = re.match(r"\**\s*([A-Za-z]+)", cells[-1])
                arch[m.group(1)] = word.group(1).upper() if word else ""
    mism = ["%s arch=%s register=%s" % (g["id"], arch.get(g["id"]), g["status"])
            for g in gates if arch.get(g["id"]) != g["status"]]
    if mism:
        bad("architecture validation Appendix E disagrees with the register: " + "; ".join(mism))
    else:
        ok("architecture validation Appendix E statuses agree with the register")

out.append("__GATES_DONE__")
print("\n".join(out))
PY
)
EOF
  # A crash in the validator must fail the gate, not silently drop its checks.
  [ "$GATES_DONE" = 1 ] || fail "gate register validator did not complete (python error above)"
fi

# ------------------------------------------------------------------ summary
printf '\n'
if [ "$FAILURES" -eq 0 ]; then
  printf 'check-docs: %s checks passed.\n' "$CHECKS"
  exit 0
fi
printf 'check-docs: %s of %s checks FAILED.\n' "$FAILURES" "$CHECKS" >&2
exit 1
