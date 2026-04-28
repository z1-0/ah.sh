use std::process;

use ah::output::print_error;

fn main() {
    // Intercept completion to also register subcommand completions.
    // clap_complete's registration only covers the top-level command.
    // `ah session <TAB>` needs the handler registered for "ah session" too.
    // let complete_shell = std::env::var("COMPLETE").ok();
    // if let Some(shell) = &complete_shell {
    //     let result = clap_complete::CompleteEnv::with_factory(ah::cli::cli)
    //         .var("COMPLETE")
    //         .try_complete(std::env::args_os(), std::env::current_dir().ok().as_deref());

    // match result {
    // Ok(true) => {
    //     // Completed — append subcommand registrations to the registration script.
    //     let stdout = io::stdout();
    //     let mut handle = stdout.lock();
    //     match shell.as_str() {
    //         "zsh" => {
    //             writeln!(handle, "compdef _clap_dynamic_completer_ah 'ah session' 'ah provider' 'ah restore' 'ah update'").unwrap();
    //         }
    //         "bash" => {
    //             for subcmd in &["session", "provider", "restore", "update"] {
    //                 writeln!(handle, "complete -F _clap_complete_ah ah {subcmd}").unwrap();
    //             }
    //         }
    //         _ => {}
    //     }
    //     return;
    // }
    // Ok(false) => { /* not a completion request, continue */ }
    // Err(e) => e.exit(),
    // }
    // }

    ah::log::with_logging(|| {
        ah::cli::init();
        ah::config::load_config()?;
        ah::cli::run()?;
        Ok(())
    })
    .unwrap_or_else(|e| {
        e.downcast_ref::<clap::Error>()
            .map(|clap_err| {
                let _ = clap_err.print();
                process::exit(clap_err.exit_code())
            })
            .unwrap_or_else(|| {
                print_error(format!("{:#}", e));
                process::exit(libc::EXIT_FAILURE)
            })
    })
}
