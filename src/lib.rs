use anyhow::Error;
use log::debug;
use util::errors::AlreadyPrintedError;
use util::indented_lines;

use crate::core::shell::Verbosity::Verbose;
use crate::{core::Shell, util::errors::InternalError};

use util::{errors::VerboseError, CliError};

pub mod core;
pub mod ops;
pub mod util;
pub mod templates;

pub fn exit_with_error(err: CliError, shell: &mut Shell) -> ! {
    debug!("exit_with_error; err={:?}", err);

    if let Some(ref err) = err.error {
        if let Some(clap_err) = err.downcast_ref::<clap::Error>() {
            let exit_code = if clap_err.use_stderr() { 1 } else { 0 };
            let _ = clap_err.print();
            std::process::exit(exit_code)
        }
    }

    let CliError { error, exit_code } = err;
    if let Some(error) = error {
        display_error(&error, shell);
    }

    std::process::exit(exit_code)
}

/// Displays an error, and all its causes, to stderr.
pub fn display_error(err: &Error, shell: &mut Shell) {
    debug!("display_error; err={:?}", err);
    _display_error(err, shell, true);
    if err
        .chain()
        .any(|e| e.downcast_ref::<InternalError>().is_some())
    {
        drop(shell.note("this is an unexpected substrate-manager internal error"));
        drop(
            shell.note(
                "we would appreciate a bug report: https://github.com/omerdn1/substrate-manager/issues/",
            ),
        );
        // TODO: Show substrate-manager cli version
        // drop(shell.note(format!("substrate {}", version())));
    }
}

fn _display_error(err: &Error, shell: &mut Shell, as_err: bool) -> bool {
    for (i, err) in err.chain().enumerate() {
        // If we're not in verbose mode then only print cause chain until one
        // marked as `VerboseError` appears.
        //
        // Generally the top error shouldn't be verbose, but check it anyways.
        if shell.verbosity() != Verbose && err.is::<VerboseError>() {
            return true;
        }
        if err.is::<AlreadyPrintedError>() {
            break;
        }
        if i == 0 {
            if as_err {
                drop(shell.error(err));
            } else {
                drop(writeln!(shell.err(), "{}", err));
            }
        } else {
            drop(writeln!(shell.err(), "\nCaused by:"));
            drop(write!(shell.err(), "{}", indented_lines(&err.to_string())));
        }
    }
    false
}
