use super::nix_parser::ShellAttrs;
use std::collections::HashMap;
use std::fmt::Write;

pub fn generate_dev_templates_flake(
    languages: &[String],
    parsed_attrs: &[(String, ShellAttrs)],
) -> String {
    let input_names: Vec<String> = languages
        .iter()
        .map(|lang| format!("dev-templates_{}", lang))
        .collect();

    let inputs_entries: Vec<String> = languages
        .iter()
        .map(|lang| {
            format!(
                "    dev-templates_{}.url = \"github:the-nix-way/dev-templates?dir={}\";\n    dev-templates_{}.inputs.nixpkgs.follows = \"nixpkgs\";",
                lang, lang, lang
            )
        })
        .collect();

    let outputs_inputs = input_names.join(", ");

    let shells_entries: Vec<String> = languages
        .iter()
        .map(|lang| {
            format!(
                "            {} = dev-templates_{}.devShells.${{system}}.default;",
                lang, lang
            )
        })
        .collect();

    // Group by attribute names to avoid duplicate keys in Nix
    let mut extra_attrs_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut env_map: HashMap<String, String> = HashMap::new();
    let parsed_attrs_by_lang: HashMap<&str, &ShellAttrs> = parsed_attrs
        .iter()
        .map(|(lang, attrs)| (lang.as_str(), attrs))
        .collect();

    // Make precedence explicit: process attributes by requested language order.
    for lang in languages {
        let Some(attrs) = parsed_attrs_by_lang.get(lang.as_str()) else {
            continue;
        };

        for (k, _) in &attrs.extra_attrs {
            // Use the 'shells' attrset defined in the Nix template
            let expr = format!("shells.\"{}\".{}", lang, k);
            extra_attrs_map.entry(k.clone()).or_default().push(expr);
        }

        for (k, _) in &attrs.env {
            let expr = format!("shells.\"{}\".{}", lang, k);
            // Keep last language in request order on conflict.
            env_map.insert(k.clone(), expr);
        }
    }

    let mut extra_attrs_str = String::new();

    // Render extra attributes in stable key order.
    let mut extra_attr_keys: Vec<&String> = extra_attrs_map.keys().collect();
    extra_attr_keys.sort_unstable();
    for key in extra_attr_keys {
        let exprs = &extra_attrs_map[key];
        if key == "postShellHook" || key == "shellHook" || key == "preHook" {
            // Concatenate hooks
            writeln!(
                extra_attrs_str,
                "            {} = {};",
                key,
                exprs.join(" + \"\\n\" + ")
            )
            .expect("writing to String cannot fail");
        } else if let Some(expr) = exprs.first() {
            // For other attributes, keep the first language in request order.
            writeln!(extra_attrs_str, "            {} = {};", key, expr)
                .expect("writing to String cannot fail");
        }
    }

    // Render env in stable key order.
    if !env_map.is_empty() {
        extra_attrs_str.push_str("            env = {\n");
        let mut env_keys: Vec<&String> = env_map.keys().collect();
        env_keys.sort_unstable();
        for key in env_keys {
            writeln!(extra_attrs_str, "              {} = {};", key, env_map[key])
                .expect("writing to String cannot fail");
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
    use super::*;

    #[test]
    fn hook_attrs_are_concatenated_in_language_order() {
        let languages = vec!["rust".to_string(), "go".to_string()];
        let parsed_attrs = vec![
            (
                "rust".to_string(),
                ShellAttrs {
                    env: Vec::new(),
                    extra_attrs: vec![
                        (
                            "postShellHook".to_string(),
                            "''echo rust post''".to_string(),
                        ),
                        ("shellHook".to_string(), "''echo rust shell''".to_string()),
                        ("preHook".to_string(), "''echo rust pre''".to_string()),
                    ],
                },
            ),
            (
                "go".to_string(),
                ShellAttrs {
                    env: Vec::new(),
                    extra_attrs: vec![
                        ("postShellHook".to_string(), "''echo go post''".to_string()),
                        ("shellHook".to_string(), "''echo go shell''".to_string()),
                        ("preHook".to_string(), "''echo go pre''".to_string()),
                    ],
                },
            ),
        ];

        let flake = generate_dev_templates_flake(&languages, &parsed_attrs);

        assert!(flake.contains(
            "postShellHook = shells.\"rust\".postShellHook + \"\\n\" + shells.\"go\".postShellHook;"
        ));
        assert!(flake.contains(
            "shellHook = shells.\"rust\".shellHook + \"\\n\" + shells.\"go\".shellHook;"
        ));
        assert!(
            flake.contains("preHook = shells.\"rust\".preHook + \"\\n\" + shells.\"go\".preHook;")
        );
    }

    #[test]
    fn env_conflict_keeps_last_language_value() {
        let languages = vec!["rust".to_string(), "go".to_string()];
        let parsed_attrs = vec![
            (
                "rust".to_string(),
                ShellAttrs {
                    env: vec![
                        ("FOO".to_string(), "\"rust\"".to_string()),
                        ("BAR".to_string(), "\"rust\"".to_string()),
                    ],
                    extra_attrs: Vec::new(),
                },
            ),
            (
                "go".to_string(),
                ShellAttrs {
                    env: vec![("FOO".to_string(), "\"go\"".to_string())],
                    extra_attrs: Vec::new(),
                },
            ),
        ];

        let flake = generate_dev_templates_flake(&languages, &parsed_attrs);

        assert!(flake.contains("BAR = shells.\"rust\".BAR;"));
        assert!(flake.contains("FOO = shells.\"go\".FOO;"));
        assert!(!flake.contains("FOO = shells.\"rust\".FOO;"));
    }
}
