pub fn providers_dir() -> String {
    // 优先级 1: 运行时环境变量 (允许运行时覆盖)
    if let Ok(dir) = std::env::var("AHSH_PROVIDERS_DIR") {
        return dir;
    }

    // 优先级 2: 编译时注入 (由 Nix 构建或开发者手动注入)
    if let Some(dir) = option_env!("AHSH_PROVIDERS_DIR") {
        return dir.to_string();
    }

    // 优先级 3: 本地开发回退
    "./providers".to_string()
}
