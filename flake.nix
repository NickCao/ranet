{
  inputs = {
    nixpkgs.url = "github:NickCao/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let pkgs = import nixpkgs { inherit system; }; in
        rec {
          devShells.default = pkgs.mkShell {
            nativeBuildInputs = with pkgs;[ rustfmt rust-analyzer ];
            inputsFrom = [ packages.default ];
          };
          packages.default = pkgs.rustPlatform.buildRustPackage {
            name = "ranet";
            src = self;
            cargoLock = {
              lockFile = ./Cargo.lock;
            };
          };
        }
      );
}
