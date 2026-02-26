use std::path::PathBuf;

pub fn get_ah_data_dir() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME environment variable not set");
    PathBuf::from(home).join(".local/share/ah")
}
