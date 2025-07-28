-- Ensure 'postgres' role exists
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_roles WHERE rolname = 'postgres'
  ) THEN
    CREATE ROLE postgres WITH LOGIN SUPERUSER;
  END IF;
END $$;

-- Ensure 'postgres' database exists
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_database WHERE datname = 'postgres'
  ) THEN
    CREATE DATABASE postgres OWNER postgres;
  END IF;
END $$;

-- Ensure 'daksha-rc' role exists
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_roles WHERE rolname = 'daksha-rc'
  ) THEN
    CREATE ROLE "daksha-rc" WITH LOGIN PASSWORD 'daksha-rc';
  END IF;
END $$;


-- Create the 'daksha-rc' database if it doesn't exist
-- This must also be run *separately* from any DO blocks.
SELECT 'CREATE DATABASE "daksha-rc" OWNER "daksha-rc"'
WHERE NOT EXISTS (SELECT 1 FROM pg_database WHERE datname = 'daksha-rc')\gexec

-- Grant all privileges on 'daksha-rc' to user
GRANT ALL PRIVILEGES ON DATABASE "daksha-rc" TO "daksha-rc";
