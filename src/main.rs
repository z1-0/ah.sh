use ah::cli::run;

fn main() {
    run().unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });
}
