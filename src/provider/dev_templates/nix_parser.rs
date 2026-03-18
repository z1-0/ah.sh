use rnix::Root;
use rnix::ast::{Apply, AttrSet, AttrpathValue, Expr};
use rowan::ast::AstNode;

#[derive(Debug, Default)]
pub struct ShellAttrs {
    /// Items meant to be env vars, e.g. env = { RUST_SRC_PATH = "..."; }
    pub env: Vec<(String, String)>,
    /// Other non-standard attributes like venvDir, postShellHook
    pub extra_attrs: Vec<(String, String)>,
}

pub fn parse_flake_shell(source: &str) -> ShellAttrs {
    let parse = Root::parse(source);
    let root = parse.tree();

    let mut shell_attrs = ShellAttrs::default();

    // Traverse the AST to find `pkgs.mkShell` or `pkgs.mkShellNoCC` calls
    for node in root.syntax().descendants() {
        if let Some(apply) = Apply::cast(node.clone())
            && is_mk_shell_call(&apply)
            && let Some(Expr::AttrSet(attr_set)) = apply.argument()
        {
            extract_attributes(&attr_set, &mut shell_attrs);
            break; // Just need the first mkShell call in the flake (the default one)
        }
    }

    shell_attrs
}

fn is_mk_shell_call(apply: &Apply) -> bool {
    let Some(lambda) = apply.lambda() else {
        return false;
    };

    let text = lambda.syntax().text().to_string();
    text.contains("mkShell") || text.contains("mkShellNoCC")
}

fn extract_attributes(attr_set: &AttrSet, shell_attrs: &mut ShellAttrs) {
    // Get all attributes in the mkShell AttrSet
    for node in attr_set.syntax().children() {
        if let Some(attrpath_value) = AttrpathValue::cast(node) {
            let Some(attrpath) = attrpath_value.attrpath() else {
                continue;
            };

            let Some(value) = attrpath_value.value() else {
                continue;
            };

            let attr_name = attrpath.to_string();
            let value_text = value.syntax().text().to_string();

            match attr_name.as_str() {
                // Ignore standard inputs that inputsFrom handles
                "packages" | "buildInputs" | "nativeBuildInputs" | "shellHook" | "inputsFrom" => {
                    continue;
                }

                // Handle nested 'env' attribute set special case
                "env" => {
                    if let Expr::AttrSet(inner_set) = value {
                        for inner_node in inner_set.syntax().children() {
                            if let Some(inner_attr) = AttrpathValue::cast(inner_node)
                                && let (Some(k), Some(v)) =
                                    (inner_attr.attrpath(), inner_attr.value())
                            {
                                shell_attrs
                                    .env
                                    .push((k.to_string(), v.syntax().text().to_string()));
                            }
                        }
                    } else {
                        // If it's not an attrset, just treat it as an extra attribute
                        shell_attrs
                            .extra_attrs
                            .push(("env".to_string(), value_text));
                    }
                }

                // Collect everything else
                _ => {
                    shell_attrs.extra_attrs.push((attr_name, value_text));
                }
            }
        }
    }
}
