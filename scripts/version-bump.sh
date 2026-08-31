#!/usr/bin/env bash
#
# version-bump.sh — consume changesets, compute the next vX.Y.Z, rewrite
# VERSION and CHANGELOG.md, and propagate the version into the documents that
# state it.
#
# Usage:
#   scripts/version-bump.sh                 # preview only, changes nothing
#   scripts/version-bump.sh --apply         # perform the release
#   scripts/version-bump.sh --set 1.0.0 --apply
#
# Version model: docs/versioning/VERSIONING.md

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHANGESET_DIR="$REPO_ROOT/.changeset"
VERSION_FILE="$REPO_ROOT/VERSION"
CHANGELOG="$REPO_ROOT/CHANGELOG.md"
ARCH_DOC="$REPO_ROOT/docs/architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md"

APPLY=0
FORCED_VERSION=""
RELEASE_DATE="$(date -u +%Y-%m-%d)"

die() { printf 'version-bump: %s\n' "$1" >&2; exit 1; }

while [ $# -gt 0 ]; do
  case "$1" in
    --apply) APPLY=1; shift ;;
    --set)   FORCED_VERSION="${2:-}"; shift 2 ;;
    --date)  RELEASE_DATE="${2:-}"; shift 2 ;;
    -h|--help)
      sed -n '2,14p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
      exit 0 ;;
    *) die "unknown argument: $1" ;;
  esac
done

[ -f "$VERSION_FILE" ] || die "missing VERSION"
[ -f "$CHANGELOG" ]    || die "missing CHANGELOG.md"

CURRENT="$(tr -d '[:space:]' < "$VERSION_FILE")"
printf '%s' "$CURRENT" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$' \
  || die "VERSION is not a bare X.Y.Z: '$CURRENT'"

CUR_MAJOR="${CURRENT%%.*}"
CUR_REST="${CURRENT#*.}"
CUR_MINOR="${CUR_REST%%.*}"
CUR_PATCH="${CUR_REST#*.}"

# ---------------------------------------------------------------- collect ---
shopt -s nullglob
CHANGESETS=()
for f in "$CHANGESET_DIR"/*.md; do
  case "$(basename "$f")" in
    README.md) continue ;;
  esac
  CHANGESETS+=("$f")
done
shopt -u nullglob

if [ "${#CHANGESETS[@]}" -eq 0 ] && [ -z "$FORCED_VERSION" ]; then
  printf 'No changesets in %s — nothing to release.\n' "${CHANGESET_DIR#"$REPO_ROOT"/}"
  exit 0
fi

# parse_field <file> <key>
parse_field() {
  awk -v key="$2" '
    NR == 1 { if ($0 != "---") { exit 1 } ; next }
    $0 == "---" { exit }
    {
      split($0, kv, ":")
      k = kv[1]
      gsub(/^[ \t]+|[ \t]+$/, "", k)
      if (k == key) {
        v = substr($0, index($0, ":") + 1)
        gsub(/^[ \t]+|[ \t]+$/, "", v)
        print v
        exit
      }
    }
  ' "$1"
}

# body <file> — everything after the closing frontmatter delimiter
parse_body() {
  awk '
    NR == 1 && $0 == "---" { infm = 1; next }
    infm && $0 == "---" { infm = 0; started = 1; next }
    started { print }
  ' "$1" | sed -e '/./,$!d'
}

HIGHEST="patch"
rank() { case "$1" in major) echo 3 ;; minor) echo 2 ;; patch) echo 1 ;; *) echo 0 ;; esac; }

declare -a E_added E_changed E_deprecated E_removed E_fixed E_security

for f in "${CHANGESETS[@]}"; do
  rel="${f#"$REPO_ROOT"/}"
  b="$(parse_field "$f" bump)"    || die "$rel: missing frontmatter"
  t="$(parse_field "$f" type)"
  c="$(parse_field "$f" component)"

  [ -n "$b" ] || die "$rel: 'bump' is required"
  [ -n "$t" ] || die "$rel: 'type' is required"
  [ -n "$c" ] || die "$rel: 'component' is required"

  case "$b" in major|minor|patch) ;; *) die "$rel: invalid bump '$b'" ;; esac
  case "$t" in added|changed|deprecated|removed|fixed|security) ;; *) die "$rel: invalid type '$t'" ;; esac

  [ "$(rank "$b")" -gt "$(rank "$HIGHEST")" ] && HIGHEST="$b"

  summary="$(parse_body "$f" | sed -n '1p')"
  [ -n "$summary" ] || die "$rel: body is empty; a changeset needs a summary line"

  entry="- **${c}** — ${summary}"
  case "$t" in
    added)      E_added+=("$entry") ;;
    changed)    E_changed+=("$entry") ;;
    deprecated) E_deprecated+=("$entry") ;;
    removed)    E_removed+=("$entry") ;;
    fixed)      E_fixed+=("$entry") ;;
    security)   E_security+=("$entry") ;;
  esac
done

# ----------------------------------------------------------------- compute ---
NOTE=""
if [ -n "$FORCED_VERSION" ]; then
  printf '%s' "$FORCED_VERSION" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$' \
    || die "--set expects a bare X.Y.Z, got '$FORCED_VERSION'"
  NEXT="$FORCED_VERSION"
  NOTE="explicitly set via --set"
else
  EFFECTIVE="$HIGHEST"
  if [ "$CUR_MAJOR" -eq 0 ] && [ "$HIGHEST" = "major" ]; then
    EFFECTIVE="minor"
    NOTE="0.x rule: a 'major' changeset bumps MINOR while MAJOR is 0. 1.0.0 is reserved for the first accepted production cutover; declare it with --set 1.0.0"
  fi
  case "$EFFECTIVE" in
    major) NEXT="$((CUR_MAJOR + 1)).0.0" ;;
    minor) NEXT="${CUR_MAJOR}.$((CUR_MINOR + 1)).0" ;;
    patch) NEXT="${CUR_MAJOR}.${CUR_MINOR}.$((CUR_PATCH + 1))" ;;
  esac
fi

# --------------------------------------------------------------- changelog ---
SECTION_FILE="$(mktemp)"
trap 'rm -f "$SECTION_FILE"' EXIT

{
  printf '## [%s] — %s\n' "$NEXT" "$RELEASE_DATE"
  emit() {
    local heading="$1"; shift
    [ "$#" -eq 0 ] && return 0
    printf '\n### %s\n\n' "$heading"
    printf '%s\n' "$@"
  }
  emit "Added"      ${E_added+"${E_added[@]}"}
  emit "Changed"    ${E_changed+"${E_changed[@]}"}
  emit "Deprecated" ${E_deprecated+"${E_deprecated[@]}"}
  emit "Removed"    ${E_removed+"${E_removed[@]}"}
  emit "Fixed"      ${E_fixed+"${E_fixed[@]}"}
  emit "Security"   ${E_security+"${E_security[@]}"}
} > "$SECTION_FILE"

# ------------------------------------------------------------------ report ---
printf 'Current version : %s\n' "$CURRENT"
printf 'Changesets      : %s\n' "${#CHANGESETS[@]}"
printf 'Highest bump    : %s\n' "$HIGHEST"
printf 'Next version    : v%s\n' "$NEXT"
[ -n "$NOTE" ] && printf 'Note            : %s\n' "$NOTE"
printf '\n--- CHANGELOG section ---\n'
cat "$SECTION_FILE"
printf -- '--- end ---\n\n'

if [ "$APPLY" -eq 0 ]; then
  printf 'Preview only. Re-run with --apply to write the release.\n'
  exit 0
fi

# ------------------------------------------------------------------- apply ---
grep -q '^## \[Unreleased\]' "$CHANGELOG" \
  || die "CHANGELOG.md has no '## [Unreleased]' anchor to insert after"

TMP="$(mktemp)"
awk -v sect="$SECTION_FILE" '
  { print }
  !done && /^## \[Unreleased\]$/ {
    print ""
    while ((getline line < sect) > 0) print line
    close(sect)
    done = 1
  }
' "$CHANGELOG" > "$TMP"
mv "$TMP" "$CHANGELOG"

printf '%s\n' "$NEXT" > "$VERSION_FILE"

# Propagate into the architecture validation header so the document can never
# disagree with VERSION (check-docs.sh enforces this).
if [ -f "$ARCH_DOC" ]; then
  TMP="$(mktemp)"
  sed -e "s/^| Specification version | v[0-9]\{1,\}\.[0-9]\{1,\}\.[0-9]\{1,\} |$/| Specification version | v${NEXT} |/" \
      "$ARCH_DOC" > "$TMP"
  mv "$TMP" "$ARCH_DOC"
fi

for f in "${CHANGESETS[@]}"; do
  rm -f "$f"
done

printf 'Released v%s.\n' "$NEXT"
printf 'Consumed %s changeset(s).\n' "${#CHANGESETS[@]}"
printf '\nNext steps:\n'
printf '  scripts/check-docs.sh\n'
printf '  git add -A && git commit -m "chore(release): v%s"\n' "$NEXT"
printf '  git tag -a v%s -m "v%s"\n' "$NEXT" "$NEXT"
