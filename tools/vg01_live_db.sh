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

# PostgreSQL represents ordinary tables as relkind='r' and partitioned table
# parents as relkind='p'. A source-level CREATE TABLE inventory contains both,
# so the live comparison must also include both. Excluding 'p' produces a
# plausible-looking off-by-one and is exactly the kind of static/live mismatch
# this gate exists to expose.
TABLE_SCOPE="n.nspname='public' AND c.relkind IN ('r','p') AND c.relname LIKE 'awcms\\_%' ESCAPE '\\' AND c.relname <> 'awcms_schema_migrations'"
TABLE_COUNT=$(scalar "SELECT count(*) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE $TABLE_SCOPE")
PARTITIONED_COUNT=$(scalar "SELECT count(*) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE $TABLE_SCOPE AND c.relkind='p'")
RLS_COUNT=$(scalar "SELECT count(*) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE $TABLE_SCOPE AND c.relrowsecurity")
FORCE_COUNT=$(scalar "SELECT count(*) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE $TABLE_SCOPE AND c.relforcerowsecurity")
GLOBAL_COUNT=$((TABLE_COUNT - FORCE_COUNT))
MIGRATION_COUNT=$(scalar "SELECT count(*) FROM awcms_schema_migrations")
POLICY_COUNT=$(scalar "SELECT count(*) FROM pg_policies WHERE schemaname='public'")

echo "VG-01 observed live state before assertions: migrations=$MIGRATION_COUNT tables=$TABLE_COUNT partitioned=$PARTITIONED_COUNT rls=$RLS_COUNT force_rls=$FORCE_COUNT global=$GLOBAL_COUNT policies=$POLICY_COUNT"

[[ "$TABLE_COUNT" -eq "$EXPECTED_TABLES" ]] || { echo "VG-01 live DB drift: expected $EXPECTED_TABLES AWCMS tables, got $TABLE_COUNT" >&2; exit 1; }
[[ "$RLS_COUNT" -eq "$EXPECTED_RLS" ]] || { echo "VG-01 live DB drift: expected $EXPECTED_RLS RLS-enabled tables, got $RLS_COUNT" >&2; exit 1; }
[[ "$FORCE_COUNT" -eq "$EXPECTED_FORCE_RLS" ]] || { echo "VG-01 live DB drift: expected $EXPECTED_FORCE_RLS FORCE-RLS tables, got $FORCE_COUNT" >&2; exit 1; }
[[ "$GLOBAL_COUNT" -eq "$EXPECTED_GLOBAL" ]] || { echo "VG-01 live DB drift: expected $EXPECTED_GLOBAL non-FORCE/global tables, got $GLOBAL_COUNT" >&2; exit 1; }
[[ "$MIGRATION_COUNT" -eq "$EXPECTED_MIGRATIONS" ]] || { echo "VG-01 live DB drift: expected $EXPECTED_MIGRATIONS applied migrations, got $MIGRATION_COUNT" >&2; exit 1; }

ROLE_COUNT=$(psql "$ADMIN_DATABASE_URL" -v ON_ERROR_STOP=1 -Atc "SELECT count(*) FROM pg_roles WHERE rolname IN ('$OWNER_ROLE','awcms_app','awcms_worker')")
[[ "$ROLE_COUNT" -eq 3 ]] || { echo "VG-01 live DB drift: expected owner + awcms_app + awcms_worker roles" >&2; exit 1; }

BAD_SECURITY_ROLES=$(psql "$ADMIN_DATABASE_URL" -v ON_ERROR_STOP=1 -Atc "SELECT count(*) FROM pg_roles WHERE rolname IN ('$OWNER_ROLE','awcms_app','awcms_worker') AND (rolsuper OR rolbypassrls)")
[[ "$BAD_SECURITY_ROLES" -eq 0 ]] || { echo "VG-01 live DB security failure: a runtime/owner role still has SUPERUSER or BYPASSRLS" >&2; exit 1; }

RUNTIME_OWNED_TABLES=$(scalar "SELECT count(*) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND c.relkind IN ('r','p') AND pg_get_userbyid(c.relowner) IN ('awcms_app','awcms_worker')")
[[ "$RUNTIME_OWNED_TABLES" -eq 0 ]] || { echo "VG-01 live DB security failure: runtime role owns business tables" >&2; exit 1; }

cat >"$OUTPUT_DIR/summary.json" <<JSON
{
  "source_repository": "ahliweb/awcms",
  "source_commit": "11f2e95a47b1328a820f976d60f978c38a067903",
  "migration_runner": "scripts/db-migrate.ts",
  "migration_count": $MIGRATION_COUNT,
  "awcms_table_count": $TABLE_COUNT,
  "partitioned_parent_count": $PARTITIONED_COUNT,
  "rls_enabled_table_count": $RLS_COUNT,
  "force_rls_table_count": $FORCE_COUNT,
  "non_force_global_table_count": $GLOBAL_COUNT,
  "policy_count": $POLICY_COUNT,
  "runtime_owned_table_count": $RUNTIME_OWNED_TABLES
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
  sha256sum migration-ledger.csv policies.csv roles.csv routine-grants.csv summary.json table-grants.csv tables-rls.csv > manifest.sha256
)

echo "VG-01 controlled live AWCMS DB introspection passed"
cat "$OUTPUT_DIR/summary.json"
cat "$OUTPUT_DIR/roles.csv"
