{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        flakePkgs = import ./pkgs { inherit pkgs; };
      in
      {
        packages = flakePkgs // {
          default = flakePkgs.rsoderh-nhg-leben-emulator;
        };

        devShell = pkgs.mkShell {
          buildInputs = flakePkgs.rsoderh-nhg-leben-emulator.propagatedBuildInputs;
        };

        formatter = pkgs.nixfmt-tree;
      }
    );
}
