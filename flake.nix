{
  description = "Simple REPL shell for untyped lambda expressions.";
  inputs = {
    nixpkgs.url = "nixpkgs";
    cf.url = "github:jzbor/cornflakes";
    cf.inputs.nixpkgs.follows = "nixpkgs";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, cf, crane }:
  cf.lib.flakeForDefaultSystems (system:
  with builtins;
  let
    pkgs = nixpkgs.legacyPackages.${system};
    craneLib = crane.lib.${system};
    nativeBuildInputs = with pkgs; [
      clang
    ];
  in {
    ### PACKAGES ###
    packages = {
      default = craneLib.buildPackage {
        pname = "lash";

        src = ./.;

        # Add extra inputs here or any other derivation settings
        # doCheck = true;
        inherit nativeBuildInputs;
      };

      docs = pkgs.stdenvNoCC.mkDerivation {
        name = "lash-docs";
        src = ./.;
        buildPhase = "${pkgs.mdbook}/bin/mdbook build book";
        installPhase = "mkdir -p $out; cp -rf book/book/* $out/";
      };
    };

    ### DEVELOPMENT SHELLS ###
    devShells.default = pkgs.mkShellNoCC {
      name = self.packages.${system}.default.name;
      nativeBuildInputs = nativeBuildInputs;
    };

    apps.open-docs = let
      port = "8080";
      script = pkgs.writeShellApplication {
        name = "open-docs";
        text = ''
          (while ! ${pkgs.lsof}/bin/lsof -i:${port} >/dev/null; do true; done; xdg-open localhost:${port}) &
          if ! ${pkgs.lsof}/bin/lsof -i:${port} >/dev/null; then
            ${pkgs.caddy}/bin/caddy file-server --listen :${port} --root "${self.packages.${system}.docs}" "$@"
          else
            echo "A server is already running"
          fi
          '';
      };
    in {
      type = "app";
      program = "${script}/bin/open-docs";
    };
  });
}

