#!/usr/bin/env bash
# VG-01: prove that the corpus checker accepts what tools/vg01_inventory.rs
# generates, before the vg01-freeze workflow publishes it.
#
# Builds a throw-away verify root = the committed corpus with frozen/static
# replaced by the freshly generated static tree (digest manifest recomputed the
# way the freeze workflow does) and `pendingArtifacts` emptied, i.e. the state
# after the freeze pull request lands, then runs tools/vg01_verify_frozen.sh on it.
# Nothing under the repository is modified.
#
# usage: tools/vg01_check_generated.sh <generated-static-dir> [corpus-root]
#   e.g. tools/vg01_check_generated.sh /tmp/vg01-a
set -euo pipefail
export LC_ALL=C

GENERATED=${1:?usage: vg01_check_generated.sh <generated-static-dir> [corpus-root]}
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "$SCRIPT_DIR/.." && pwd)
CORPUS=${2:-$REPO_ROOT/contracts/legacy/awcms}

[[ -f "$GENERATED/manifest.json" ]] || { echo "not a generated static tree: $GENERATED" >&2; exit 1; }
[[ -d "$CORPUS/frozen/live" ]] || { echo "committed corpus has no frozen/live: $CORPUS" >&2; exit 1; }

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
VERIFY_ROOT="$WORK/root"
mkdir -p "$VERIFY_ROOT/frozen/static"

cp -a "$CORPUS/frozen/live" "$VERIFY_ROOT/frozen/live"
cp "$CORPUS/frozen/provenance.json" "$VERIFY_ROOT/frozen/provenance.json"
cp -a "$CORPUS/authorization" "$VERIFY_ROOT/authorization"
cp "$CORPUS/manifest.json" "$VERIFY_ROOT/manifest.json"
cp -a "$GENERATED/." "$VERIFY_ROOT/frozen/static/"
(
  cd "$VERIFY_ROOT/frozen/static"
  find . -type f ! -name manifest.sha256 -print0 | sort -z | xargs -0 sha256sum > "$WORK/manifest.sha256"
  mv "$WORK/manifest.sha256" manifest.sha256
)

# The freeze pull request empties pendingArtifacts; verify that end state.
awk '
  # A one-line array ("pendingArtifacts": [...]) is replaced without skipping;
  # only a multi-line array needs its following lines dropped.
  /"pendingArtifacts": \[/ { print "  \"pendingArtifacts\": [],"; if ($0 !~ /\]/) skipping = 1; next }
  skipping == 1 { if ($0 ~ /^  \],?$/) { skipping = 0 } ; next }
  { print }
' "$CORPUS/fixture-usage.json" > "$VERIFY_ROOT/fixture-usage.json"
if ! grep -Fq '"pendingArtifacts": [],' "$VERIFY_ROOT/fixture-usage.json"; then
  echo "could not empty pendingArtifacts in the verify copy" >&2
  exit 1
fi

bash "$SCRIPT_DIR/vg01_verify_frozen.sh" "$VERIFY_ROOT"
echo "VG-01 generated static tree validates as the post-freeze corpus"
