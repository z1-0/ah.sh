use ah::output::print_error;

fn main() {
    ah::log::with_logging(|| {
        ah::config::load_config()?;
        ah::cli::run()?;
        Ok(())
    })
    .unwrap_or_else(|e| {
        e.downcast_ref::<clap::Error>()
            .map(|clap_err| {
                let _ = clap_err.print();
                std::process::exit(clap_err.exit_code())
            })
            .unwrap_or_else(|| {
                print_error(format!("{:#}", e));
                std::process::exit(libc::EXIT_FAILURE)
            })
    })
}
