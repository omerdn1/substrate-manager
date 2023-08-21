use clap::{Arg, ArgAction, Command};

pub trait AppExt: Sized {
    fn _arg(self, arg: Arg) -> Self;

    fn arg_quiet(self) -> Self {
        self._arg(flag("quiet", "Do not print cargo log messages").short('q'))
    }
}

impl AppExt for Command {
    fn _arg(self, arg: Arg) -> Self {
        self.arg(arg)
    }
}

pub fn subcommand(name: &'static str) -> Command {
    Command::new(name)
    // App::new(name)
    //     .dont_collapse_args_in_usage(true)
    //     .setting(AppSettings::DeriveDisplayOrder)
}

pub fn opt(name: &'static str, help: &'static str) -> Arg {
    Arg::new(name).long(name).help(help).action(ArgAction::Set)
}

pub fn flag(name: &'static str, help: &'static str) -> Arg {
    Arg::new(name)
        .long(name)
        .help(help)
        .action(ArgAction::SetTrue)
}

#[track_caller]
pub fn ignore_unknown<T: Default>(r: Result<T, clap::parser::MatchesError>) -> T {
    match r {
        Ok(t) => t,
        Err(clap::parser::MatchesError::UnknownArgument { .. }) => Default::default(),
        Err(e) => {
            panic!("Mismatch between definition and access: {}", e);
        }
    }
}
