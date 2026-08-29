#!/usr/bin/env bash
set -euo pipefail
export LC_ALL=C

if [[ $# -ne 5 ]]; then
  echo "usage: vg01_live_db.sh <awcms-checkout> <admin-url> <owner-url> <target-admin-url> <output-dir>" >&2
  exit 2
fi

AWCMS_DIR=$1
ADMIN_DATABASE_URL=$2
OWNER_DATABASE_URL=$3
TARGET_ADMIN_DATABASE_URL=$4
OUTPUT_DIR=$5

DB_NAME=awcms_vg01
OWNER_ROLE=awcms_vg01_owner
OWNER_PASSWORD=awcms_vg01_owner_fixture_password

EXPECTED_TABLES=152
EXPECTED_RLS=134
EXPECTED_FORCE_RLS=134
EXPECTED_GLOBAL=18
EXPECTED_MIGRATIONS=148

rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"

# Match the pinned AWCMS integration harness posture: the schema owner is
# temporarily SUPERUSER only while the real migration runner executes, because
# historical AWCMS migrations configure custom GUC defaults on runtime roles.
# It is demoted immediately after migrations so FORCE RLS can be inspected in
# the same owner posture exercised by the AWCMS integration harness.
psql "$ADMIN_DATABASE_URL" -v ON_ERROR_STOP=1 <<SQL
DROP DATABASE IF EXISTS "$DB_NAME" WITH (FORCE);
DROP ROLE IF EXISTS "$OWNER_ROLE";
CREATE ROLE "$OWNER_ROLE" LOGIN SUPERUSER PASSWORD '$OWNER_PASSWORD';
CREATE DATABASE "$DB_NAME" OWNER "$OWNER_ROLE";
SQL

(
  cd "$AWCMS_DIR"
  DATABASE_URL="$OWNER_DATABASE_URL" bun run db:migrate
)

psql "$ADMIN_DATABASE_URL" -v ON_ERROR_STOP=1 <<SQL
ALTER ROLE "$OWNER_ROLE" NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;
SQL

scalar() {
  psql "$TARGET_ADMIN_DATABASE_URL" -v ON_ERROR_STOP=1 -Atc "$1"
}

# Compare the pinned AWCMS source's OWN cumulative table/RLS derivation with the
# database produced by the pinned AWCMS migration runner. This avoids creating
# a second parser whose bugs could be mistaken for product drift.
(
  cd "$AWCMS_DIR"
  bun -e '
    import { loadMigrations } from "./scripts/lib/migrations.ts";
    import { deriveTableRlsStates } from "./scripts/lib/table-rls-states.ts";
    for (const row of deriveTableRlsStates(loadMigrations())) {
      console.log(`${row.table}\t${row.rowLevelSecurity ? "t" : "f"}\t${row.force ? "t" : "f"}`);
    }
  '
) | sort >"$OUTPUT_DIR/source-tables-rls.tsv"
cut -f1 "$OUTPUT_DIR/source-tables-rls.tsv" >"$OUTPUT_DIR/source-table-names.txt"

# PostgreSQL ordinary tables are relkind='r'; partitioned table parents are
# relkind='p'. A source-level CREATE TABLE inventory can represent either.
TABLE_SCOPE="n.nspname='public' AND c.relkind IN ('r','p') AND c.relname LIKE 'awcms\\_%' ESCAPE '\\' AND c.relname <> 'awcms_schema_migrations'"

psql "$TARGET_ADMIN_DATABASE_URL" -v ON_ERROR_STOP=1 -AtF $'\t' -c "SELECT c.relname, CASE WHEN c.relrowsecurity THEN 't' ELSE 'f' END, CASE WHEN c.relforcerowsecurity THEN 't' ELSE 'f' END FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE $TABLE_SCOPE ORDER BY c.relname" >"$OUTPUT_DIR/live-tables-rls.tsv"
cut -f1 "$OUTPUT_DIR/live-tables-rls.tsv" >"$OUTPUT_DIR/live-table-names.txt"

comm -23 "$OUTPUT_DIR/source-table-names.txt" "$OUTPUT_DIR/live-table-names.txt" >"$OUTPUT_DIR/source-only-tables.txt"
comm -13 "$OUTPUT_DIR/source-table-names.txt" "$OUTPUT_DIR/live-table-names.txt" >"$OUTPUT_DIR/live-only-tables.txt"

diff -u "$OUTPUT_DIR/source-tables-rls.tsv" "$OUTPUT_DIR/live-tables-rls.tsv" >"$OUTPUT_DIR/source-vs-live-tables.diff" || true

SOURCE_TABLE_COUNT=$(wc -l <"$OUTPUT_DIR/source-table-names.txt" | tr -d ' ')
SOURCE_RLS_COUNT=$(awk -F '\t' '$2 == "t" { count += 1 } END { print count + 0 }' "$OUTPUT_DIR/source-tables-rls.tsv")
SOURCE_FORCE_COUNT=$(awk -F '\t' '$3 == "t" { count += 1 } END { print count + 0 }' "$OUTPUT_DIR/source-tables-rls.tsv")
SOURCE_GLOBAL_COUNT=$((SOURCE_TABLE_COUNT - SOURCE_FORCE_COUNT))

TABLE_COUNT=$(scalar "SELECT count(*) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE $TABLE_SCOPE")
PARTITIONED_COUNT=$(scalar "SELECT count(*) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE $TABLE_SCOPE AND c.relkind='p'")
RLS_COUNT=$(scalar "SELECT count(*) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE $TABLE_SCOPE AND c.relrowsecurity")
FORCE_COUNT=$(scalar "SELECT count(*) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE $TABLE_SCOPE AND c.relforcerowsecurity")
GLOBAL_COUNT=$((TABLE_COUNT - FORCE_COUNT))
MIGRATION_COUNT=$(scalar "SELECT count(*) FROM awcms_schema_migrations")
POLICY_COUNT=$(scalar "SELECT count(*) FROM pg_policies WHERE schemaname='public'")

ROLE_COUNT=$(psql "$ADMIN_DATABASE_URL" -v ON_ERROR_STOP=1 -Atc "SELECT count(*) FROM pg_roles WHERE rolname IN ('$OWNER_ROLE','awcms_app','awcms_worker')")
BAD_SECURITY_ROLES=$(psql "$ADMIN_DATABASE_URL" -v ON_ERROR_STOP=1 -Atc "SELECT count(*) FROM pg_roles WHERE rolname IN ('$OWNER_ROLE','awcms_app','awcms_worker') AND (rolsuper OR rolbypassrls)")
RUNTIME_OWNED_TABLES=$(scalar "SELECT count(*) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND c.relkind IN ('r','p') AND pg_get_userbyid(c.relowner) IN ('awcms_app','awcms_worker')")

cat >"$OUTPUT_DIR/summary.json" <<JSON
{
  "source_repository": "ahliweb/awcms",
  "source_commit": "11f2e95a47b1328a820f976d60f978c38a067903",
  "migration_runner": "scripts/db-migrate.ts",
  "migration_count": $MIGRATION_COUNT,
  "source_derived_table_count": $SOURCE_TABLE_COUNT,
  "source_derived_rls_count": $SOURCE_RLS_COUNT,
  "source_derived_force_rls_count": $SOURCE_FORCE_COUNT,
  "source_derived_non_force_count": $SOURCE_GLOBAL_COUNT,
  "live_awcms_table_count": $TABLE_COUNT,
  "live_partitioned_parent_count": $PARTITIONED_COUNT,
  "live_rls_enabled_table_count": $RLS_COUNT,
  "live_force_rls_table_count": $FORCE_COUNT,
  "live_non_force_global_table_count": $GLOBAL_COUNT,
  "policy_count": $POLICY_COUNT,
  "runtime_owned_table_count": $RUNTIME_OWNED_TABLES,
  "role_count": $ROLE_COUNT,
  "bad_security_role_count": $BAD_SECURITY_ROLES
}
JSON

psql "$ADMIN_DATABASE_URL" -v ON_ERROR_STOP=1 -c "COPY (SELECT rolname, rolsuper, rolbypassrls, rolcreaterole, rolcreatedb, rolcanlogin FROM pg_roles WHERE rolname IN ('$OWNER_ROLE','awcms_app','awcms_worker') ORDER BY rolname) TO STDOUT WITH (FORMAT CSV, HEADER TRUE)" >"$OUTPUT_DIR/roles.csv"

psql "$TARGET_ADMIN_DATABASE_URL" -v ON_ERROR_STOP=1 -c "COPY (SELECT c.relname AS table_name, c.relkind, pg_get_userbyid(c.relowner) AS owner, c.relrowsecurity AS rls_enabled, c.relforcerowsecurity AS force_rls, (SELECT count(*) FROM pg_policy p WHERE p.polrelid=c.oid) AS policy_count FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND c.relkind IN ('r','p') AND c.relname LIKE 'awcms\\_%' ESCAPE '\\' ORDER BY c.relname) TO STDOUT WITH (FORMAT CSV, HEADER TRUE)" >"$OUTPUT_DIR/tables-rls.csv"

psql "$TARGET_ADMIN_DATABASE_URL" -v ON_ERROR_STOP=1 -c "COPY (SELECT tablename, policyname, permissive, roles::text, cmd, COALESCE(qual,''), COALESCE(with_check,'') FROM pg_policies WHERE schemaname='public' ORDER BY tablename, policyname) TO STDOUT WITH (FORMAT CSV, HEADER TRUE)" >"$OUTPUT_DIR/policies.csv"

psql "$TARGET_ADMIN_DATABASE_URL" -v ON_ERROR_STOP=1 -c "COPY (SELECT grantee, table_name, privilege_type, is_grantable FROM information_schema.table_privileges WHERE table_schema='public' AND grantee IN ('awcms_app','awcms_worker') ORDER BY grantee, table_name, privilege_type) TO STDOUT WITH (FORMAT CSV, HEADER TRUE)" >"$OUTPUT_DIR/table-grants.csv"

psql "$TARGET_ADMIN_DATABASE_URL" -v ON_ERROR_STOP=1 -c "COPY (SELECT grantee, routine_name, privilege_type, is_grantable FROM information_schema.routine_privileges WHERE specific_schema='public' AND grantee IN ('awcms_app','awcms_worker') ORDER BY grantee, routine_name, privilege_type) TO STDOUT WITH (FORMAT CSV, HEADER TRUE)" >"$OUTPUT_DIR/routine-grants.csv"

psql "$TARGET_ADMIN_DATABASE_URL" -v ON_ERROR_STOP=1 -c "COPY (SELECT migration_name, checksum FROM awcms_schema_migrations ORDER BY migration_name) TO STDOUT WITH (FORMAT CSV, HEADER TRUE)" >"$OUTPUT_DIR/migration-ledger.csv"

(
  cd "$OUTPUT_DIR"
  sha256sum \
    live-only-tables.txt live-table-names.txt live-tables-rls.tsv \
    migration-ledger.csv policies.csv roles.csv routine-grants.csv \
    source-only-tables.txt source-table-names.txt source-tables-rls.tsv \
    source-vs-live-tables.diff summary.json table-grants.csv tables-rls.csv \
    > manifest.sha256
)

echo "VG-01 source-derived state: tables=$SOURCE_TABLE_COUNT rls=$SOURCE_RLS_COUNT force_rls=$SOURCE_FORCE_COUNT global=$SOURCE_GLOBAL_COUNT"
echo "VG-01 live state: migrations=$MIGRATION_COUNT tables=$TABLE_COUNT partitioned=$PARTITIONED_COUNT rls=$RLS_COUNT force_rls=$FORCE_COUNT global=$GLOBAL_COUNT policies=$POLICY_COUNT"

if [[ -s "$OUTPUT_DIR/source-only-tables.txt" ]]; then
  echo "VG-01 source-only tables:" >&2
  sed 's/^/  - /' "$OUTPUT_DIR/source-only-tables.txt" >&2
fi
if [[ -s "$OUTPUT_DIR/live-only-tables.txt" ]]; then
  echo "VG-01 live-only tables:" >&2
  sed 's/^/  - /' "$OUTPUT_DIR/live-only-tables.txt" >&2
fi
if [[ -s "$OUTPUT_DIR/source-vs-live-tables.diff" ]]; then
  echo "VG-01 source/live table-RLS diff:" >&2
  cat "$OUTPUT_DIR/source-vs-live-tables.diff" >&2
fi

[[ "$SOURCE_TABLE_COUNT" -eq "$EXPECTED_TABLES" ]] || { echo "VG-01 pinned source inventory drift: expected $EXPECTED_TABLES tables, got $SOURCE_TABLE_COUNT" >&2; exit 1; }
[[ "$SOURCE_RLS_COUNT" -eq "$EXPECTED_RLS" ]] || { echo "VG-01 pinned source inventory drift: expected $EXPECTED_RLS RLS tables, got $SOURCE_RLS_COUNT" >&2; exit 1; }
[[ "$SOURCE_FORCE_COUNT" -eq "$EXPECTED_FORCE_RLS" ]] || { echo "VG-01 pinned source inventory drift: expected $EXPECTED_FORCE_RLS FORCE-RLS tables, got $SOURCE_FORCE_COUNT" >&2; exit 1; }
[[ "$SOURCE_GLOBAL_COUNT" -eq "$EXPECTED_GLOBAL" ]] || { echo "VG-01 pinned source inventory drift: expected $EXPECTED_GLOBAL non-FORCE tables, got $SOURCE_GLOBAL_COUNT" >&2; exit 1; }
[[ "$MIGRATION_COUNT" -eq "$EXPECTED_MIGRATIONS" ]] || { echo "VG-01 live DB drift: expected $EXPECTED_MIGRATIONS applied migrations, got $MIGRATION_COUNT" >&2; exit 1; }
[[ "$TABLE_COUNT" -eq "$SOURCE_TABLE_COUNT" ]] || { echo "VG-01 source/live mismatch: source derives $SOURCE_TABLE_COUNT tables, live PostgreSQL has $TABLE_COUNT" >&2; exit 1; }
[[ "$RLS_COUNT" -eq "$SOURCE_RLS_COUNT" ]] || { echo "VG-01 source/live mismatch: source derives $SOURCE_RLS_COUNT RLS tables, live PostgreSQL has $RLS_COUNT" >&2; exit 1; }
[[ "$FORCE_COUNT" -eq "$SOURCE_FORCE_COUNT" ]] || { echo "VG-01 source/live mismatch: source derives $SOURCE_FORCE_COUNT FORCE-RLS tables, live PostgreSQL has $FORCE_COUNT" >&2; exit 1; }
[[ ! -s "$OUTPUT_DIR/source-only-tables.txt" && ! -s "$OUTPUT_DIR/live-only-tables.txt" ]] || { echo "VG-01 source/live table-name set mismatch" >&2; exit 1; }
[[ ! -s "$OUTPUT_DIR/source-vs-live-tables.diff" ]] || { echo "VG-01 source/live RLS-state mismatch" >&2; exit 1; }
[[ "$ROLE_COUNT" -eq 3 ]] || { echo "VG-01 live DB drift: expected owner + awcms_app + awcms_worker roles" >&2; exit 1; }
[[ "$BAD_SECURITY_ROLES" -eq 0 ]] || { echo "VG-01 live DB security failure: a runtime/owner role still has SUPERUSER or BYPASSRLS" >&2; exit 1; }
[[ "$RUNTIME_OWNED_TABLES" -eq 0 ]] || { echo "VG-01 live DB security failure: runtime role owns business tables" >&2; exit 1; }

echo "VG-01 controlled live AWCMS DB introspection passed"
cat "$OUTPUT_DIR/summary.json"
cat "$OUTPUT_DIR/roles.csv"
