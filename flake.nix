{
  description = "A very basic flake";

  outputs = {
    self,
    nixpkgs,
  }: let
    buildInputs = [];
    nativeBuildInputs = [pkgs.rustc pkgs.cargo];
    pkgs = import nixpkgs {system = "x86_64-linux";};
  in {
    devShells.x86_64-linux.default = pkgs.mkShell {
      packages = [
        pkgs.rust-analyzer
      ];
      inherit buildInputs nativeBuildInputs;
    };
  };
}
