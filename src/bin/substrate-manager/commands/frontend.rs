use clap::{ArgMatches, Command};
use inquire::Confirm;
use substrate_manager::ops;
use substrate_manager::ops::substrate_frontend::clone_substrate_frontend_template;
use substrate_manager::util::config::{ChainInfo, ProjectType};
use substrate_manager::util::{command_prelude::*, CliResult, Config};

use super::GlobalContext;

pub fn cli() -> Command {
    subcommand("frontend")
        .about("Run the frontend application that connects to the node")
        .arg_quiet()
        .after_help("Run `substrate help frontend` for more detailed information.\n")
}

pub fn exec(_config: &mut Config, _args: &ArgMatches) -> CliResult {
    ops::frontend()?;

    Ok(())
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = GlobalContext)]
#[interactive_clap(output_context = FrontendContext)]
pub struct Frontend;

#[derive(Debug, Clone)]
pub struct FrontendContext;

impl FrontendContext {
    pub fn from_previous_context(
        previous_context: GlobalContext,
        _scope: &<Frontend as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        if let ProjectType::Chain(ChainInfo { frontend_path, .. }) =
            &previous_context.config.project_type.clone().unwrap()
        {
            if !frontend_path.exists() {
                println!("Could not locate your frontend directory.");
                let is_generate = Confirm::new("Would you like to generate it? (y/n)").prompt()?;
                if !is_generate {
                    return Ok(Self);
                }
                if let Err(e) = clone_substrate_frontend_template(frontend_path) {
                    return Err(color_eyre::eyre::eyre!(e));
                }
            }
            if let Err(e) = ops::frontend() {
                return Err(color_eyre::eyre::eyre!(e));
            }

            Ok(Self)
        } else {
            color_eyre::eyre::bail!("Incorrect project type")
        }
    }
}
