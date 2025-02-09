{
    inputs = {
        nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
        flake-utils.url = "github:numtide/flake-utils";
        rust-overlay = {
            url = "github:oxalica/rust-overlay";
            inputs = {
                nixpkgs.follows = "nixpkgs";
            };
        };
        crane.url = "github:ipetkov/crane";
    };
    outputs = { self, nixpkgs, flake-utils, rust-overlay, crane }:
        flake-utils.lib.eachDefaultSystem (system:
            let
                pkgs = import nixpkgs {
                    inherit system;
                    overlays = [ (import rust-overlay) ];
                };

                craneLib = (crane.mkLib pkgs).overrideToolchain (p: p.rust-bin.nightly.latest.default);

                sqlFilter = path: _type: null != builtins.match ".*sql$" path;
                sqlxFilter = path: _type: null != builtins.match ".*sqlx/.*.json$" path;
                sqlOrCargo = path: type: (sqlFilter path type) || (sqlxFilter path type) || (craneLib.filterCargoSources path type);

                crateExpression = { darwin
                  , postgresql
                  , exiftool
                  , openssl
                  , libiconv
                  , lib
                  , pkg-config
                  , curl
                  , cacert
                  , stdenv }:
                    craneLib.buildPackage {
                        src = lib.cleanSourceWith {
                            src = craneLib.path ./.; # The original, unfiltered source
                            filter = sqlOrCargo;
                        };

                        # Dependencies which need to be build for the current platform
                        # on which we are doing the cross compilation. In this case,
                        # pkg-config needs to run on the build platform so that the build
                        # script can find the location of openssl. Note that we don't
                        # need to specify the rustToolchain here since it was already
                        # overridden above.
                        nativeBuildInputs = [
                            pkg-config
                            curl
                            cacert
                        ] ++ lib.optionals stdenv.buildPlatform.isDarwin [
                            libiconv darwin.apple_sdk.frameworks.SystemConfiguration
                        ];

                        # Dependencies which need to be built for the platform on which
                        # the binary will run. In this case, we need to compile openssl
                        # so that it can be linked with our executable.
                        buildInputs = [
                            postgresql
                            exiftool
                            openssl
                        ];

                        # Tell cargo about the linker and an optional emulater. So they can be used in `cargo build`
                        # and `cargo run`.
                        # Environment variables are in format `CARGO_TARGET_<UPPERCASE_UNDERSCORE_RUST_TRIPLE>_LINKER`.
                        # They are also be set in `.cargo/config.toml` instead.
                        # See: https://doc.rust-lang.org/cargo/reference/config.html#target
#                        CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = "aarch64-unknown-linux-gnu-gcc";
#                        CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER = "qemu-aarch64";

                        # Tell cargo which target we want to build (so it doesn't default to the build system).
                        # We can either set a cargo flag explicitly with a flag or with an environment variable.
#                        cargoExtraArgs = "--target aarch64-unknown-linux-gnu";
                        # CARGO_BUILD_TARGET = "aarch64-unknown-linux-gnu";

                        # This environment variable may be necessary if any of your dependencies use a
                        # build-script which invokes the `cc` crate to build some other code. The `cc` crate
                        # should automatically pick up on our target-specific linker above, but this may be
                        # necessary if the build script needs to compile and run some extra code on the build
                        # system.
#                        HOST_CC = "${stdenv.cc.nativePrefix}cc";
                    };

                # Assuming the above expression was in a file called myCrate.nix
                # this would be defined as:
                # my-crate = pkgs.callPackage ./myCrate.nix { };
                bin = pkgs.callPackage crateExpression { };

                dockerImage = pkgs.dockerTools.buildImage {
                    name = "photonic";
                    tag = "latest";
                    created = "now";
                    copyToRoot = [ bin pkgs.exiftool ];
                    config = {
                        Cmd = [ "${bin}/bin/photonic" ];
                    };
                };
            in
            with pkgs;
            {
                packages = {
                    inherit bin dockerImage;
                    default = bin;
                };
                devShells.default = mkShell {
                    inputsFrom = [ bin ];
                };
            }
        );
}