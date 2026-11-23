#!/usr/bin/env bash
# VG-01: recompute the SHA-256 of every AWCMS migration file in a pinned checkout
# and compare it with the frozen migrations.json. Closes the "per-file SHA-256
# compared with the SQL itself" criterion of issue #3: the corpus checker can
# only prove the digests are well formed and ordered; only a checkout holds the
# SQL they were taken from.
#
# usage: tools/vg01_verify_migrations.sh <migrations.json> <awcms-checkout>
#   e.g. tools/vg01_verify_migrations.sh \
#          contracts/legacy/awcms/frozen/static/migrations.json .legacy/awcms
#
# Fails when a listed file is missing, a digest differs, the count field is
# wrong, a listed path is unsafe, or the checkout holds a migration that the
# frozen list does not (or the reverse).
set -euo pipefail
export LC_ALL=C

MANIFEST=${1:?usage: vg01_verify_migrations.sh <migrations.json> <awcms-checkout>}
CHECKOUT=${2:?usage: vg01_verify_migrations.sh <migrations.json> <awcms-checkout>}

[[ -f "$MANIFEST" ]] || { echo "VG-01 migrations manifest missing: $MANIFEST" >&2; exit 1; }
[[ -d "$CHECKOUT/sql" ]] || { echo "VG-01 AWCMS checkout has no sql/ directory: $CHECKOUT" >&2; exit 1; }

declared=$(grep -oE '"count": [0-9]+' "$MANIFEST" | head -n 1 | grep -oE '[0-9]+' || true)
[[ -n "$declared" ]] || { echo "VG-01 migrations manifest has no count field" >&2; exit 1; }

mapfile -t entries < <(grep -oE '"path": "[^"]+", "sha256": "[0-9a-f]{64}"' "$MANIFEST")
if [[ "${#entries[@]}" -ne "$declared" ]]; then
  echo "VG-01 migrations manifest lists ${#entries[@]} entries but declares count $declared" >&2
  exit 1
fi

listed=$(mktemp)
actual=$(mktemp)
trap 'rm -f "$listed" "$actual"' EXIT

mismatches=0
for entry in "${entries[@]}"; do
  path=${entry#\"path\": \"}
  path=${path%%\"*}
  want=${entry##*\"sha256\": \"}
  want=${want%\"}
  if [[ ! "$path" =~ ^sql/[A-Za-z0-9._-]+\.sql$ ]]; then
    echo "VG-01 unsafe or unexpected migration path in manifest: $path" >&2
    exit 1
  fi
  echo "$path" >> "$listed"
  file="$CHECKOUT/$path"
  if [[ ! -f "$file" ]]; then
    echo "VG-01 migration listed but absent from the checkout: $path" >&2
    mismatches=$((mismatches + 1))
    continue
  fi
  got=$(sha256sum "$file" | cut -d' ' -f1)
  if [[ "$got" != "$want" ]]; then
    echo "VG-01 migration SHA-256 differs from its SQL: $path (manifest $want, file $got)" >&2
    mismatches=$((mismatches + 1))
  fi
done

(cd "$CHECKOUT" && find sql -maxdepth 1 -type f -name '*.sql' | sort) > "$actual"
sort -o "$listed" "$listed"
if ! diff -u "$listed" "$actual" >&2; then
  echo "VG-01 migration set in the checkout differs from the frozen list" >&2
  mismatches=$((mismatches + 1))
fi

if [[ "$mismatches" -ne 0 ]]; then
  echo "VG-01 migration verification failed: $mismatches problem(s)" >&2
  exit 1
fi
echo "VG-01 verified SHA-256 of ${#entries[@]} migration files against the SQL in $CHECKOUT"
