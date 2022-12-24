{
  description = "TODO";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, advisory-db, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        inherit (pkgs) lib;
        craneLib = crane.lib.${system};
        buildInputs = with pkgs; [
        ];
        nativeBuildInputs = with pkgs; [
          # Programs and libraries used at *build-time*
        ];
        toolchain = with pkgs; [
          cargo
          rustc
          rustfmt
          clippy
          rust-analyzer
          cargo-limit
          nixpkgs-fmt 
          yq
          ripgrep
        ] ++ nativeBuildInputs ++ buildInputs;

        # Common derivation arguments used for all builds
        commonArgs = {
          src = ./.;
          buildInputs = buildInputs;
          nativeBuildInputs = nativeBuildInputs;
        };

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          # Additional arguments specific to this derivation can be added here.
          # Be warned that using `//` will not do a deep copy of nested
          # structures
          pname = "nstow";
        });

        # Run clippy (and deny all warnings) on the crate source,
        # resuing the dependency artifacts (e.g. from build scripts or
        # proc-macros) from above.
        #
        # Note that this is done as a separate derivation so it
        # does not impact building just the crate by itself.
        myCrateClippy = craneLib.cargoClippy (commonArgs // {
          # Again we apply some extra arguments only to this derivation
          # and not every where else. In this case we add some clippy flags
          inherit cargoArtifacts;
          cargoClippyExtraArgs = "--all-targets -- --deny warnings";
        });

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        myCrate = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });

        # Also run the crate tests under cargo-tarpaulin so that we can keep
        # track of code coverage
        myCrateCoverage = craneLib.cargoTarpaulin (commonArgs // {
          inherit cargoArtifacts;
        });

        myCrateDoc = craneLib.cargoDoc (commonArgs // {
          inherit cargoArtifacts;
        });

        myCrateFormat = craneLib.cargoFmt (commonArgs // {
          inherit cargoArtifacts;
        });

        myCrateAudit = craneLib.cargoAudit (commonArgs // {
          inherit cargoArtifacts advisory-db;
        });

        # Run tests with cargo-nextest
        myCrateNextest = craneLib.cargoNextest (commonArgs // {
          inherit cargoArtifacts;
          partitions = 1;
          partitionType = "count";
        });
 
      in
      {
        packages = {
          crate = myCrate;
          fmt = myCrateFormat;
          clippy = myCrateClippy;
          audit = myCrateAudit;
          doc = myCrateDoc;
        };

        packages.default = myCrate;

        checks = {
          inherit
           # Build the crate as part of `nix flake check` for convenience
           myCrate
           myCrateFormat
           myCrateClippy
           #myCrateAudit
           myCrateDoc;
        #} // lib.optionalAttrs (system == "x86_64-linux") {
        #    myCrateCoverage = craneLib.cargoTarpaulin {
        #      inherit cargoArtifacts;
        #    };
        };

        apps.default = flake-utils.lib.mkApp {
          drv = myCrate;
        };

        devShells.default = pkgs.mkShell {
          # Tools that should be avaliable in the shell
          nativeBuildInputs = toolchain;
        };
      });
}
