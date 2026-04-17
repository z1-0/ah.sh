use std::process::ExitCode;

use ah::output::print_error;

fn main() -> ExitCode {
    ah::config::load_config()
        .and_then(|_| ah::cli::run())
        .map(|_| ExitCode::SUCCESS)
        .unwrap_or_else(|e| {
            e.downcast_ref::<clap::Error>()
                .map(|clap_err| {
                    let _ = clap_err.print();
                    ExitCode::from(clap_err.exit_code() as u8)
                })
                .unwrap_or_else(|| {
                    print_error(format!("{:#}", e));
                    ExitCode::FAILURE
                })
        })
}
