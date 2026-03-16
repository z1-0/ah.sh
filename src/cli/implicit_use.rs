use clap::Command;
use std::ffi::OsString;

use crate::providers::is_maybe_language;

pub fn maybe_implicit_use_command(mut args: Vec<OsString>, cmd: &Command) -> Vec<OsString> {
    let first_arg = match args.get(1) {
        Some(arg) => arg.to_string_lossy(),
        None => return args,
    };
    let first_arg_str = first_arg.as_ref();

    if is_known_command(cmd, first_arg_str) || is_top_level_flag(cmd, first_arg_str) {
        return args;
    }

    if should_implicit_use_command(cmd, first_arg_str) {
        args.insert(1, OsString::from("use"));
    }

    args
}

fn is_known_command(cmd: &Command, arg: &str) -> bool {
    cmd.get_subcommands()
        .any(|s| s.get_name() == arg || s.get_all_aliases().any(|alias| alias == arg))
}

fn is_top_level_flag(cmd: &Command, arg: &str) -> bool {
    cmd.get_arguments()
        .any(|a| matches_flag(arg, a.get_short(), a.get_long()))
}

fn should_implicit_use_command(cmd: &Command, arg: &str) -> bool {
    is_maybe_language(arg) || is_use_command_flag(cmd, arg)
}

fn is_use_command_flag(cmd: &Command, arg: &str) -> bool {
    cmd.find_subcommand("use").is_some_and(|s| {
        s.get_arguments()
            .any(|a| matches_flag(arg, a.get_short(), a.get_long()))
    })
}

fn matches_flag(arg: &str, short: Option<char>, long: Option<&str>) -> bool {
    short.is_some_and(|s| arg == format!("-{s}")) || long.is_some_and(|l| arg == format!("--{l}"))
}
