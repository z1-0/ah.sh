use ah::cli::languages;

fn main() {
    languages().unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });
}
