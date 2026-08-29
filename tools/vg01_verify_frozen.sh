#!/usr/bin/env bash
set -euo pipefail
export LC_ALL=C

ROOT=${1:-contracts/legacy/awcms}
STATIC="$ROOT/frozen/static"
LIVE="$ROOT/frozen/live"
VECTORS="$ROOT/authorization/vectors.json"

required=(
  "$STATIC/manifest.json"
  "$STATIC/manifest.sha256"
  "$STATIC/modules.json"
  "$STATIC/migrations.json"
  "$STATIC/astro-consumer.json"
  "$STATIC/source/awcms-public-api.openapi.yaml"
  "$STATIC/source/awcms-domain-events.asyncapi.yaml"
  "$LIVE/manifest.sha256"
  "$LIVE/summary.json"
  "$LIVE/roles.csv"
  "$LIVE/tables-rls.csv"
  "$LIVE/policies.csv"
  "$LIVE/table-grants.csv"
  "$LIVE/routine-grants.csv"
  "$LIVE/migration-ledger.csv"
  "$VECTORS"
)
for path in "${required[@]}"; do
  [[ -f "$path" ]] || { echo "VG-01 frozen evidence missing: $path" >&2; exit 1; }
done

(
  cd "$STATIC"
  sha256sum -c manifest.sha256
)
(
  cd "$LIVE"
  sha256sum -c manifest.sha256
)

[[ ! -s "$LIVE/source-only-tables.txt" ]] || { echo "VG-01 source-only table drift exists" >&2; exit 1; }
[[ ! -s "$LIVE/live-only-tables.txt" ]] || { echo "VG-01 live-only table drift exists" >&2; exit 1; }
[[ ! -s "$LIVE/source-vs-live-tables.diff" ]] || { echo "VG-01 source/live RLS-state drift exists" >&2; exit 1; }

grep -Fq '11f2e95a47b1328a820f976d60f978c38a067903' "$STATIC/manifest.json"
grep -Fq '7b753be619244541b817d5d8e7d3b72cfe88d4f9' "$STATIC/manifest.json"
grep -Fq '"migration_count": 148' "$LIVE/summary.json"
grep -Fq '"live_all_awcms_table_count": 152' "$LIVE/summary.json"
grep -Fq '"live_business_table_count_excluding_migration_ledger": 151' "$LIVE/summary.json"
grep -Fq '"live_rls_enabled_table_count": 134' "$LIVE/summary.json"
grep -Fq '"live_force_rls_table_count": 134' "$LIVE/summary.json"
grep -Fq '"runtime_owned_table_count": 0' "$LIVE/summary.json"
grep -Fq '"bad_security_role_count": 0' "$LIVE/summary.json"

for id in \
  ASTRO-TENANT-001 ASTRO-TENANT-002 ASTRO-TENANT-003 ASTRO-TENANT-004 ASTRO-TENANT-005 \
  NEWSLETTER-PUBLIC-001 NEWSLETTER-PUBLIC-002 NEWSLETTER-PUBLIC-003 NEWSLETTER-PUBLIC-004 \
  NEWSLETTER-CONSENT-001 NEWSLETTER-TOKEN-001 NEWSLETTER-UNSUBSCRIBE-001 \
  DB-RLS-001 DB-ROLE-001; do
  grep -Fq "\"id\": \"$id\"" "$VECTORS" || { echo "VG-01 required vector missing: $id" >&2; exit 1; }
done

# The frozen corpus must remain a fixture, never an executable dependency.
if find "$ROOT/frozen" -type l -print -quit | grep -q .; then
  echo "VG-01 frozen corpus contains symlinks" >&2
  exit 1
fi

echo "VG-01 frozen static/live evidence and authorization vector register verified"
