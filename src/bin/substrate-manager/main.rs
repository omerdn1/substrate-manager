use clap::Command;
use commands::{
    CliContractCmd, CliMissingProjectCmd, ContractCmd, MissingProjectCmd,
    ChainCmd, CliChainCmd, GlobalContext,
};
use substrate_manager::{
    core::Shell,
    util::{config::ProjectType, errors::CliResult, Config},
};

mod commands;

fn main() {
    let mut config = LazyConfig::new();
    // TODO: Proxy locking for config

    let result = cli(&mut config);

    match result {
        Err(e) => {
            // TODO: Use shell across entire lifetime, not only for errors
            let mut shell = Shell::new();
            substrate_manager::exit_with_error(e, &mut shell)
        }
        Ok(()) => {}
    }
}

fn cli(config: &mut LazyConfig) -> CliResult {
    // let cli = Command::new("substrate")
    //     // Overrides default help output
    //     .help_template(
    //         "\
    // Substrate package manager

    // Usage: {usage}

    // Commands:
    // {subcommands}

    // Options:
    // {options}

    // See 'substrate help <command>' for more information on a specific command.\n",
    //     )
    //     .subcommands(commands::builtin());
    // let args = cli.clone().try_get_matches()?;

    // CAUTION: Be careful with using `config` until it is configured below.
    // let config = config.get();

    // let (cmd, subcommand_args) = match args.subcommand() {
    //     Some((cmd, args)) => (cmd, args),
    //     _ => {
    //         // No subcommand provided.
    //         // cli.clone().print_help()?;
    //         commands::main(config, &args);
    //         return Ok(());
    //     }
    // };

    // if let Some(exec) = commands::builtin_exec(cmd) {
    //     return commands::main(config, subcommand_args);
    // }
    //

    let config = config.get();
    let global_context = GlobalContext { config: config.clone() };

    match config.project_type() {
        Some(ProjectType::Chain(_)) => {
            commands::main::<ChainCmd, CliChainCmd>(global_context.to_owned())?
        }
        Some(ProjectType::Contract(_)) => {
            commands::main::<ContractCmd, CliContractCmd>(global_context.to_owned())?
        }
        None => commands::main::<MissingProjectCmd, CliMissingProjectCmd>(global_context.to_owned())?,
    }
    // commands::main(config)?;

    Ok(())
}

/// Delay loading [`Config`] until access.
///
/// In the common path, the [`Config`] is dependent on CLI parsing and shouldn't be loaded until
/// after that is done but some other paths (like fix or earlier errors) might need access to it,
/// so this provides a way to share the instance and the implementation across these different
/// accesses.
#[derive(Debug)]
pub struct LazyConfig {
    config: Option<Config>,
}

impl LazyConfig {
    pub fn new() -> Self {
        Self { config: None }
    }

    /// Check whether the config is loaded
    ///
    /// This is useful for asserts in case the environment needs to be setup before loading
    pub fn is_init(&self) -> bool {
        self.config.is_some()
    }

    /// Get the config, loading it if needed
    ///
    /// On error, the process is terminated
    pub fn get(&mut self) -> &Config {
        self.get_mut()
    }

    /// Get the config, loading it if needed
    ///
    /// On error, the process is terminated
    pub fn get_mut(&mut self) -> &mut Config {
        self.config.get_or_insert_with(|| match Config::default() {
            Ok(cfg) => cfg,
            Err(e) => {
                let mut shell = Shell::new();
                substrate_manager::exit_with_error(e.into(), &mut shell)
            }
        })
    }
}

// #[test]
// fn verify_cli() {
//     cli().debug_assert();
// }
