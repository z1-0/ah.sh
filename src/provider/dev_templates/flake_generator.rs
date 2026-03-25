use super::nix_parser::ShellAttrs;
use std::collections::HashMap;
use std::fmt::Write;

/// Generates a dev-templates flake.nix that combines multiple language shells.
///
/// # Parameters
/// - `languages`: Deduplicated language names in requested order
/// - `parsed_attrs`: Shell attributes for each language at the same index
///
/// # Important
/// The two slices must have the same length, and `parsed_attrs[i]` must
/// correspond to `languages[i]`. This is enforced by the caller in `mod.rs`.
pub fn generate_dev_templates_flake(languages: &[String], parsed_attrs: &[ShellAttrs]) -> String {
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

    // Make precedence explicit: process attributes by requested language order.
    for (i, lang) in languages.iter().enumerate() {
        let Some(attrs) = parsed_attrs.get(i) else {
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
