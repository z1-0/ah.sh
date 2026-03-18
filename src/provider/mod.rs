use self::registry::into_shell_provider;

pub mod dev_templates;
pub mod devenv;
pub mod language_maps;
pub mod registry;
pub mod types;

pub use language_maps::{
    is_maybe_language, language_map_for_display, language_map_for_provider,
    map_language_for_provider,
};
pub use registry::{ProviderInfo, all_providers, provider_info, provider_name};
pub use types::{
    EnsureFilesResult, ProviderKeyOrAll, ProviderType, ShellProvider, validate_languages,
};

impl ProviderType {
    pub fn into_shell_provider(self) -> Box<dyn ShellProvider> {
        into_shell_provider(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::error::AppError;
    use crate::provider::{
        ProviderType, all_providers, is_maybe_language, language_map_for_display,
        map_language_for_provider, provider_info,
    };

    #[test]
    fn maps_real_alias_for_each_provider() {
        assert_eq!(
            map_language_for_provider("devenv", "js").unwrap(),
            "javascript"
        );
        assert_eq!(
            map_language_for_provider("dev-templates", "ts").unwrap(),
            "node"
        );
    }

    #[test]
    fn display_map_omits_self_aliases_and_sorts_aliases() {
        let devenv_map = language_map_for_display("devenv").unwrap();
        assert_eq!(
            devenv_map.get("javascript").unwrap(),
            &vec!["js".to_string()]
        );

        let dev_templates_map = language_map_for_display("dev-templates").unwrap();
        assert_eq!(
            dev_templates_map.get("node").unwrap(),
            &vec![
                "javascript".to_string(),
                "js".to_string(),
                "ts".to_string(),
                "typescript".to_string(),
            ]
        );
    }

    #[test]
    fn detects_known_aliases_as_possible_languages() {
        assert!(is_maybe_language("js"));
        assert!(!is_maybe_language("totally-not-a-language"));
    }

    #[test]
    fn invalid_provider_name_preserves_current_error_shape() {
        let err = map_language_for_provider("not-a-provider", "js").unwrap_err();

        match err {
            AppError::Generic(message) => {
                assert!(message.contains("Unsupported provider: not-a-provider"));
            }
            other => panic!("expected AppError::Generic, got {other:?}"),
        }
    }

    #[test]
    fn provider_registry_exposes_stable_order_and_names() {
        let providers = all_providers();
        assert_eq!(providers.len(), 2);
        assert_eq!(providers[0].name(), "devenv");
        assert_eq!(providers[1].name(), "dev-templates");
    }

    #[test]
    fn provider_info_exposes_metadata_accessors() {
        let info = provider_info(ProviderType::Devenv);
        assert_eq!(info.name(), "devenv");
        assert_eq!(info.normalize_language("js").unwrap(), "javascript");
        assert!(
            info.supported_languages()
                .unwrap()
                .contains(&"javascript".to_string())
        );
        assert_eq!(
            info.display_language_map()
                .unwrap()
                .get("javascript")
                .unwrap(),
            &vec!["js".to_string()]
        );
    }
}
