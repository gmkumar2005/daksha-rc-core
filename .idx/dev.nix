{ pkgs, ... }:

let
  rust-overlay = import (builtins.fetchTarball {
    url = "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
  });

  pkgs_with_rust = import pkgs.path {
    system = pkgs.system;
    overlays = [ rust-overlay ];
  };

  rustToolchain = pkgs_with_rust.rust-bin.stable."1.86.0".default;
in
{
  channel = "stable-25.05";

  packages = [
    rustToolchain
    pkgs.cargo-make
    pkgs.sqlx-cli
    pkgs.just
    pkgs.postgresql
    pkgs.gcc
    pkgs.openssl
    pkgs.pkg-config
    pkgs.perl
  ];

  env = {
    OPENSSL_DIR = "${pkgs.openssl}";
    OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
    OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
    PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
    LD_LIBRARY_PATH = "${pkgs.openssl.out}/lib";
    DATABASE_URL= "postgres://daksha-rc:daksha-rc@localhost:5432/daksha-rc";
  };

  services.postgres.enable = true;

  idx = {
    extensions = [ ];
    previews = {
      enable = true;
      previews = { };
    };
    workspace = {
      onCreate = { };

      onStart = {
        init-postgres = ''
          export PGDATA=$PWD/pgdata
          export PGHOST=/tmp/postgres
          export PGPORT=5432

          mkdir -p "$PGDATA"

          if [ ! -f "$PGDATA/PG_VERSION" ]; then
            echo "🔧 Initializing PostgreSQL cluster..."
            initdb -D "$PGDATA" --auth=trust --username=postgres
          fi

          echo "🚀 Starting PostgreSQL..."
          pg_ctl -D "$PGDATA" -o "-k $PGHOST" -w start

          echo "📦 Seeding roles and databases..."

          cat > init.sql <<EOF
DO \$\$
BEGIN
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'postgres') THEN
    CREATE ROLE postgres WITH LOGIN SUPERUSER;
  END IF;
END \$\$;

DO \$\$
BEGIN
  IF NOT EXISTS (SELECT FROM pg_database WHERE datname = 'postgres') THEN
    CREATE DATABASE postgres OWNER postgres;
  END IF;
END \$\$;
EOF

          psql -h "$PGHOST" -d postgres -f init.sql
        '';
      };
    };
  };
}
 