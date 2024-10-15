{
  inputs = {
    nixpkgs.url = "github:NickCao/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ]
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ self.overlays.default ];
          };
        in
        rec {
          packages = {
            default = pkgs.ranet;
            ranet = pkgs.ranet;
            ranet-static = pkgs.pkgsStatic.ranet;
          };
          devShells.default = pkgs.mkShell {
            nativeBuildInputs = with pkgs;[ rustfmt rust-analyzer ];
            inputsFrom = [ packages.default ];
          };
        }
      ) // {
      overlays.default = final: _: with final; {
        ranet = rustPlatform.buildRustPackage {
          name = "ranet";
          src = self;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          checkFlags = [ "--skip=address::test::remote" ];
        };
      };
    };
}
