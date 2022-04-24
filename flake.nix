{
  inputs = {
    nixpkgs.url = "github:NickCao/nixpkgs/nixos-unstable-small";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let pkgs = import nixpkgs { inherit system; };
        in
        {
          devShell = pkgs.mkShell {
            nativeBuildInputs = with pkgs;[ cargo rustc rustfmt rust-analyzer ];
          };
        }
      );
}
