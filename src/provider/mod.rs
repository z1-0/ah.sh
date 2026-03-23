use self::registry::into_shell_provider;

pub mod dev_templates;
pub mod devenv;
pub mod language_maps;
pub mod registry;
pub mod types;

pub use language_maps::{is_maybe_language, language_map_for_display, map_language_for_provider};
pub use registry::{ProviderInfo, all_providers, provider_info, provider_name};
pub use types::{ProviderKeyOrAll, ProviderType, ShellProvider, validate_languages};

impl ProviderType {
    pub fn into_shell_provider(self) -> Box<dyn ShellProvider> {
        into_shell_provider(self)
    }
}
