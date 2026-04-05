{
  description = "Inkwell - AI-powered novel writing application";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = pkgs.rust-bin.stable.latest.default;
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            rust-analyzer
            cargo-watch
            cargo-edit

            cargo-tauri

            nodejs
            pnpm
            typescript
            typescript-language-server

            pkg-config
            openssl
            webkitgtk_4_1
            libsoup_3
            gtk3
            librsvg
            wrapGAppsHook4
            glib-networking
            at-spi2-atk
          ];

          shellHook = ''
            export LD_LIBRARY_PATH="${
              pkgs.lib.makeLibraryPath (
                with pkgs;
                [
                  webkitgtk_4_1
                  libsoup_3
                  gtk3
                  glib
                  at-spi2-atk
                  cairo
                  pango
                  gdk-pixbuf
                  librsvg
                  openssl
                ]
              )
            }:$LD_LIBRARY_PATH"
            export XDG_DATA_DIRS="${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:$XDG_DATA_DIRS"
          '';
        };
      }
    );
}
