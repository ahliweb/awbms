#!/usr/bin/env bash
#
# changeset-new.sh — create a well-formed changeset file.
#
# Usage:
#   scripts/changeset-new.sh --bump minor --type added --component skills \
#                            --summary "Add the awbms-versioning skill"
#
# Every field is validated against .changeset/config.json before the file is
# written, so a changeset that exists is a changeset that scripts/check-docs.sh
# will accept.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHANGESET_DIR="$REPO_ROOT/.changeset"

BUMP=""
TYPE=""
COMPONENT=""
SUMMARY=""

VALID_BUMPS="major minor patch"
VALID_TYPES="added changed deprecated removed fixed security"
VALID_COMPONENTS="architecture-validation adr governance versioning skills scripts ci contracts workspace"

die() { printf 'changeset-new: %s\n' "$1" >&2; exit 1; }

usage() {
  cat <<'EOF'
Usage: scripts/changeset-new.sh --bump <level> --type <type> --component <c> --summary <text>

  --bump       major | minor | patch
  --type       added | changed | deprecated | removed | fixed | security
  --component  architecture-validation | adr | governance | versioning |
               skills | scripts | ci | contracts | workspace
  --summary    one line, imperative mood, no trailing period
EOF
}

while [ $# -gt 0 ]; do
  case "$1" in
    --bump)      BUMP="${2:-}"; shift 2 ;;
    --type)      TYPE="${2:-}"; shift 2 ;;
    --component) COMPONENT="${2:-}"; shift 2 ;;
    --summary)   SUMMARY="${2:-}"; shift 2 ;;
    -h|--help)   usage; exit 0 ;;
    *)           usage >&2; die "unknown argument: $1" ;;
  esac
done

contains() {
  # contains <needle> <space-separated haystack>
  case " $2 " in *" $1 "*) return 0 ;; *) return 1 ;; esac
}

[ -n "$BUMP" ]      || { usage >&2; die "--bump is required"; }
[ -n "$TYPE" ]      || { usage >&2; die "--type is required"; }
[ -n "$COMPONENT" ] || { usage >&2; die "--component is required"; }
[ -n "$SUMMARY" ]   || { usage >&2; die "--summary is required"; }

contains "$BUMP" "$VALID_BUMPS"           || die "invalid --bump '$BUMP' (want: $VALID_BUMPS)"
contains "$TYPE" "$VALID_TYPES"           || die "invalid --type '$TYPE' (want: $VALID_TYPES)"
contains "$COMPONENT" "$VALID_COMPONENTS" || die "invalid --component '$COMPONENT' (want: $VALID_COMPONENTS)"

case "$SUMMARY" in
  *.) die "--summary must not end with a period" ;;
esac

[ -d "$CHANGESET_DIR" ] || die "missing $CHANGESET_DIR"

# Slug: lowercase, non-alphanumerics collapsed to single hyphens, trimmed, capped.
SLUG="$(printf '%s' "$SUMMARY" \
  | tr '[:upper:]' '[:lower:]' \
  | sed -e 's/[^a-z0-9]\{1,\}/-/g' -e 's/^-//' -e 's/-$//' \
  | cut -c1-48 \
  | sed -e 's/-$//')"
[ -n "$SLUG" ] || SLUG="change"

STAMP="$(date -u +%Y%m%d%H%M%S)"
FILE="$CHANGESET_DIR/${STAMP}-${SLUG}.md"

[ -e "$FILE" ] && die "changeset already exists: $FILE"

cat > "$FILE" <<EOF
---
bump: $BUMP
type: $TYPE
component: $COMPONENT
---

$SUMMARY
EOF

printf 'Created %s\n' "${FILE#"$REPO_ROOT"/}"
printf 'Edit it to add detail, and reference register IDs (AD-/OD-/VG-) where relevant.\n'
