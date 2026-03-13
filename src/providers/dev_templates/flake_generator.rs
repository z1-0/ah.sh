use super::nix_parser::ShellAttrs;
use std::collections::HashMap;

pub fn generate_dev_templates_flake(
    languages: &[String],
    parsed_attrs: &[(String, ShellAttrs)],
) -> String {

    let inputs_from: Vec<String> = languages
        .iter()
        .map(|lang| format!("\"{}\"", lang))
        .collect();

    // Group by attribute names to avoid duplicate keys in Nix
    let mut extra_attrs_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut env_map: HashMap<String, String> = HashMap::new();

    for (lang, attrs) in parsed_attrs {
        for (k, _) in &attrs.extra_attrs {
            // Use the 'shells' attrset defined in the Nix template
            let expr = format!("shells.\"{}\".{}", lang, k);
            extra_attrs_map.entry(k.clone()).or_default().push(expr);
        }

        for (k, _) in &attrs.env {
            let expr = format!("shells.\"{}\".{}", lang, k);
            // Just take the last one if there's a conflict
            env_map.insert(k.clone(), expr);
        }
    }

    let mut extra_attrs_str = String::new();

    // Render extra attributes
    for (k, exprs) in extra_attrs_map {
        if k == "postShellHook" || k == "shellHook" || k == "preHook" {
            // Concatenate hooks
            extra_attrs_str.push_str(&format!(
                "            {} = {};\n",
                k,
                exprs.join(" + \"\\n\" + ")
            ));
        } else {
            // For other attributes, just use the first one if there are conflicts
            extra_attrs_str.push_str(&format!("            {} = {};\n", k, exprs[0]));
        }
    }

    // Render env
    if !env_map.is_empty() {
        extra_attrs_str.push_str("            env = {\n");
        for (k, expr) in env_map {
            extra_attrs_str.push_str(&format!("              {} = {};\n", k, expr));
        }
        extra_attrs_str.push_str("            };\n");
    }

    format!(
        r#"{{
  inputs.nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1";

  outputs =
    {{ nixpkgs, ... }}:
    let
      allSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forAllSystems =
        f: nixpkgs.lib.genAttrs allSystems (system: f {{ pkgs = import nixpkgs {{ inherit system; }}; }});

      getTemplateShell =
        system: lang:
        (builtins.getFlake "github:the-nix-way/dev-templates?dir=${{lang}}").devShells.${{system}}.default;
    in
    {{
      devShells = forAllSystems (
        {{ pkgs }}:
        let
          languages = [
            {}
          ];
          shells = nixpkgs.lib.genAttrs languages (lang: getTemplateShell pkgs.stdenv.hostPlatform.system lang);
          inputsFrom = builtins.attrValues shells;
        in
        {{
          default = pkgs.mkShellNoCC {{
            inherit inputsFrom;
{}          }};
        }}
      );
    }};
}}
"#,
        inputs_from.join("\n            "),
        extra_attrs_str
    )
}

#[cfg(test)]
mod tests {
    use super::{generate_dev_templates_flake, ShellAttrs};

    #[test]
    fn generate_dev_templates_flake_includes_languages_and_attrs() {
        let languages = vec!["rust".to_string(), "go".to_string()];
        let parsed_attrs = vec![(
            "rust".to_string(),
            ShellAttrs {
                env: vec![("FOO".to_string(), "\"bar\"".to_string())],
                extra_attrs: vec![("venvDir".to_string(), "\".venv\"".to_string())],
            },
        )];

        let flake = generate_dev_templates_flake(&languages, &parsed_attrs);

        assert!(
            flake.contains("\"rust\"") && flake.contains("\"go\""),
            "expected flake to include both languages, got:\n{flake}"
        );

        // Since parsed_attrs contains both env and extra attrs, assert both are rendered (no OR).
        assert!(
            flake.contains("env = {"),
            "expected flake to include env attrset, got:\n{flake}"
        );
        assert!(
            flake.contains("FOO = shells.\"rust\".FOO;"),
            "expected flake env to reference shells for FOO, got:\n{flake}"
        );
        assert!(
            flake.contains("venvDir = shells.\"rust\".venvDir;"),
            "expected flake to include venvDir extra attr, got:\n{flake}"
        );
    }
}
