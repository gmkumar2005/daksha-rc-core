{ pkgs, ... }:

let
  # Import the oxalica/rust-overlay
  # It's highly recommended to pin this to a specific commit for reproducibility
  # Example:
  # rust-overlay = import (builtins.fetchTarball {
  #   url = "https://github.com/oxalica/rust-overlay/archive/0d4c7b8c7b8c7b8c7b8c7b8c7b8c7b8c7b8c7b8c.tar.gz"; # Replace with a real commit hash
  #   sha256 = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="; # Replace with the actual sha256
  # });
  # For now, sticking with master for easier copy-paste, but be aware:
  rust-overlay = import (builtins.fetchTarball {
    url = "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
  });

  # Apply the overlay to your pkgs set
  pkgs_with_rust = import pkgs.path {
    system = pkgs.system;
    overlays = [ rust-overlay ];
  };

  # Define the specific Rust toolchain version you want
  rustToolchain = pkgs_with_rust.rust-bin.stable."1.86.0".default;

in
{
  # Which nixpkgs channel to use.
  channel = "stable-24.05"; # or "unstable"

  # Use https://search.nixos.org/packages to find packages
  packages = [
    rustToolchain # This includes rustc, cargo, rustfmt, and clippy
    pkgs.cargo-make
    pkgs.sqlx-cli
    pkgs.just
    pkgs.postgresql
    pkgs.gcc        # C compiler/linker
    pkgs.openssl    # OpenSSL runtime library
    pkgs.pkg-config # Essential for build scripts to find C libraries
    pkgs.perl       # openssl's build system might sometimes need perl
  ];

  # Sets environment variables in the workspace
  env = {
    # Explicitly tell openssl-sys where to find OpenSSL's headers and libs for compilation
    # This points to the 'dev' output of the openssl package in the Nix store.
    OPENSSL_DIR = "${pkgs.openssl.dev}";

    # Ensure pkg-config knows where to look for openssl's .pc files
    PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

    # Sometimes, particularly if openssl-sys is trying to link dynamically,
    # it needs to know where the shared libraries are at runtime (during its build script execution).
    # This is often a fallback, but can be crucial.
    LD_LIBRARY_PATH = "${pkgs.openssl.dev}/lib"; # For Linux environments
    # If target is macOS, you might also need:
    # DYLD_LIBRARY_PATH = "${pkgs.openssl.dev}/lib";
  };
  idx = {
    extensions = [
      # "vscodevim.vim"
    ];

    previews = {
      enable = true;
      previews = {
        # web = {
        #   command = ["npm" "run" "dev"];
        #   manager = "web";
        #   env = {
        #     PORT = "$PORT";
        #   };
        # };
      };
    };

    workspace = {
      onCreate = {
        # npm-install = "npm install";
      };
      onStart = {
        # watch-backend = "npm run watch-backend";
      };
    };
  };
}