use tracing_attributes::instrument;

#[instrument(skip_all, fields(provider = "devenv", languages = ?languages))]
pub fn generate_devenv_flake(languages: &[String]) -> String {
    let languages_enable_str = languages
        .iter()
        .map(|lang| format!("languages.{}.enable = true;", lang))
        .collect::<Vec<_>>()
        .join("\n                ");

    format!(
        r#"{{
  inputs = {{
    nixpkgs.url = "github:cachix/devenv-nixpkgs/rolling";
    devenv.url = "github:cachix/devenv";
  }};

  nixConfig = {{
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw= cachix.cachix.org-1:eWNHQldwUO7G2VkjpnjDbWwy4KQ/HNxht7H4SSoMckM=";
    extra-substituters = "https://devenv.cachix.org https://cachix.cachix.org";
  }};

  outputs =
    {{ nixpkgs, devenv, ... }}@inputs:
    let
      inherit (nixpkgs) lib;

      allSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forAllSystems =
        f: lib.genAttrs allSystems (system: f {{ pkgs = import nixpkgs {{ inherit system; }}; }});
    in
    {{
      devShells = forAllSystems (
        {{ pkgs }}:
        {{
          default = devenv.lib.mkShell {{
            inherit inputs pkgs;
            modules = [
              {{
                {}
              }}
            ];
          }};
        }}
      );
    }};
}}
"#,
        languages_enable_str
    )
}
