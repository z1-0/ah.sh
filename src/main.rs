fn main() {
    if let Err(e) = ah::cli::run() {
        eprintln!("\x1b[1;31merror:\x1b[0m {e}");
        std::process::exit(1);
    }
}
