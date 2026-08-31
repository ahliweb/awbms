\set ON_ERROR_STOP on

DROP TABLE IF EXISTS awbms_vg04_probe;
DROP ROLE IF EXISTS awbms_app_vg04;

CREATE ROLE awbms_app_vg04
  LOGIN
  PASSWORD 'awbms_vg04_ci_password'
  NOSUPERUSER
  NOCREATEDB
  NOCREATEROLE
  NOINHERIT
  NOBYPASSRLS;

CREATE TABLE awbms_vg04_probe (
  id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  tenant_id text NOT NULL,
  payload text NOT NULL
);

ALTER TABLE awbms_vg04_probe ENABLE ROW LEVEL SECURITY;
ALTER TABLE awbms_vg04_probe FORCE ROW LEVEL SECURITY;

CREATE POLICY awbms_vg04_probe_tenant_isolation
  ON awbms_vg04_probe
  USING (tenant_id = current_setting('app.current_tenant_id', true))
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

GRANT SELECT, INSERT, UPDATE, DELETE ON awbms_vg04_probe TO awbms_app_vg04;
GRANT USAGE, SELECT ON SEQUENCE awbms_vg04_probe_id_seq TO awbms_app_vg04;

INSERT INTO awbms_vg04_probe (tenant_id, payload)
VALUES
  ('tenant-a', 'visible-only-to-a'),
  ('tenant-b', 'visible-only-to-b');
