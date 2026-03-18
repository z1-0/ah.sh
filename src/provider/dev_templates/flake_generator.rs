use super::nix_parser::ShellAttrs;
use std::collections::HashMap;

pub fn generate_dev_templates_flake(
    languages: &[String],
    parsed_attrs: &[(String, ShellAttrs)],
) -> String {
    let input_names: Vec<String> = languages
        .iter()
        .map(|lang| format!("tmpl_{}", lang))
        .collect();

    let inputs_entries: Vec<String> = languages
        .iter()
        .map(|lang| {
            format!(
                "  tmpl_{}.url = \"github:the-nix-way/dev-templates?dir={}\";",
                lang, lang
            )
        })
        .collect();

    let outputs_inputs = input_names.join(", ");

    let shells_entries: Vec<String> = languages
        .iter()
        .map(|lang| format!("          {} = tmpl_{}.devShells.${{system}}.default;", lang, lang))
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
  inputs = {{
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1";
{}
  }};

  outputs =
    {{ nixpkgs, {}, ... }}:
    let
      allSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forAllSystems =
        f: nixpkgs.lib.genAttrs allSystems (system: f {{ pkgs = import nixpkgs {{ inherit system; }}; }});
    in
    {{
      devShells = forAllSystems (
        {{ pkgs }}:
        let
          system = pkgs.stdenv.hostPlatform.system;
          shells = {{
{}
          }};
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
        inputs_entries.join("\n"),
        outputs_inputs,
        shells_entries.join("\n"),
        extra_attrs_str
    )
}

#[cfg(test)]
mod tests {
    use super::generate_dev_templates_flake;

    #[test]
    fn dev_templates_flake_uses_explicit_inputs_single() {
        let langs = vec!["rust".to_string()];
        let flake = generate_dev_templates_flake(&langs, &[]);
        assert!(flake.contains("tmpl_rust.url"));
        assert!(!flake.contains("builtins.getFlake"));
    }

    #[test]
    fn dev_templates_flake_outputs_include_all_inputs() {
        let langs = vec!["rust".to_string(), "python".to_string()];
        let flake = generate_dev_templates_flake(&langs, &[]);
        assert!(flake.contains("{ nixpkgs, tmpl_rust, tmpl_python, ... }"));
    }
}
