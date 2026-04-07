use std::process::ExitCode;

fn main() -> ExitCode {
    match ah_sh::cli::run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            if let Some(clap_err) = e.downcast_ref::<clap::Error>() {
                if let Err(print_err) = clap_err.print() {
                    eprintln!("{print_err}");
                }
                return ExitCode::from(clap_err.exit_code() as u8);
            }

            eprintln!("{:#}", e);
            ExitCode::from(1)
        }
    }
}
