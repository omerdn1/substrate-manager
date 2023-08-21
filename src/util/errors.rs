use core::fmt;

use anyhow::{anyhow, Error};
use color_eyre::Report;

pub type SubstrateResult<T> = anyhow::Result<T>;

// =============================================================================
// Verbose error

/// An error wrapper for errors that should only be displayed with `--verbose`.
///
/// This should only be used in rare cases. When emitting this error, you
/// should have a normal error higher up the error-cause chain (like "could
/// not compile `foo`"), so at least *something* gets printed without
/// `--verbose`.
pub struct VerboseError {
    inner: Error,
}

impl VerboseError {
    pub fn new(inner: Error) -> VerboseError {
        VerboseError { inner }
    }
}

impl std::error::Error for VerboseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}

impl fmt::Debug for VerboseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl fmt::Display for VerboseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

// =============================================================================
// Internal error

/// An unexpected, internal error.
///
/// This should only be used for unexpected errors. It prints a message asking
/// the user to file a bug report.
pub struct InternalError {
    inner: Error,
}

impl InternalError {
    pub fn new(inner: Error) -> InternalError {
        InternalError { inner }
    }
}

impl std::error::Error for InternalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}

impl fmt::Debug for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl fmt::Display for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

// =============================================================================
// Already printed error

/// An error that does not need to be printed because it does not add any new
/// information to what has already been printed.
pub struct AlreadyPrintedError {
    inner: Error,
}

impl AlreadyPrintedError {
    pub fn new(inner: Error) -> Self {
        AlreadyPrintedError { inner }
    }
}

impl std::error::Error for AlreadyPrintedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}

impl fmt::Debug for AlreadyPrintedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl fmt::Display for AlreadyPrintedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

// =============================================================================
// CLI errors

pub type CliResult = Result<(), CliError>;

#[derive(Debug)]
/// The CLI error is the error type used at Substrate's CLI-layer.
///
/// All errors from the lib side of Substrate will get wrapped with this error.
/// Other errors (such as command-line argument validation) will create this
/// directly.
pub struct CliError {
    pub error: Option<anyhow::Error>,
    /// The process exit code.
    pub exit_code: i32,
}

impl CliError {
    pub fn new(error: anyhow::Error, code: i32) -> CliError {
        CliError {
            error: Some(error),
            exit_code: code,
        }
    }

    pub fn code(code: i32) -> CliError {
        CliError {
            error: None,
            exit_code: code,
        }
    }
}

impl From<anyhow::Error> for CliError {
    fn from(err: anyhow::Error) -> CliError {
        CliError::new(err, 101)
    }
}

impl From<clap::Error> for CliError {
    fn from(err: clap::Error) -> CliError {
        let code = if err.use_stderr() { 1 } else { 0 };
        CliError::new(err.into(), code)
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> CliError {
        CliError::new(err.into(), 1)
    }
}

impl From<Report> for CliError {
    fn from(err: color_eyre::Report) -> CliError {
        CliError {
            error: Some(anyhow!(err)),
            exit_code: 1,
        }
    }
}
