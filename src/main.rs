use std::process::ExitCode;

fn main() -> ExitCode {
    match ah::cli::run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            // Detect CLI usage error for exit code 2
            // Check if error message contains CLI usage indicators
            let exit_code = if e.to_string().contains("Usage:")
                || e.to_string().contains("error:")
                || e.to_string().contains("unexpected argument")
                || e.to_string().contains("found unexpected argument")
            {
                2
            } else {
                1
            };
            eprintln!("{:#}", e);
            ExitCode::from(exit_code as u8)
        }
    }
}
