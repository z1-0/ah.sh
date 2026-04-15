use std::process::ExitCode;

fn main() -> ExitCode {
    println!("{:?}", ah::config::get());

    ah::cli::run()
        .map(|_| ExitCode::SUCCESS)
        .unwrap_or_else(|e| {
            e.downcast_ref::<clap::Error>()
                .map(|clap_err| {
                    let _ = clap_err.print();
                    ExitCode::from(clap_err.exit_code() as u8)
                })
                .unwrap_or_else(|| {
                    eprintln!("{:#}", e);
                    ExitCode::FAILURE
                })
        })
}
