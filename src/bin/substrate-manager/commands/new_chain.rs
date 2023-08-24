use std::path::Path;

use clap::{Arg, ArgAction, ArgMatches, Command};
use inquire::{Select, Text};

use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};
use substrate_manager::ops;
use substrate_manager::ops::substrate_new::NewOptions;
use substrate_manager::util::{normalize_paths, command_prelude::*, CliResult, Config};

use super::GlobalContext;

pub fn cli() -> Command {
    subcommand("new_chain")
        .about("Create a new substrate chain project at <path>")
        .arg_quiet()
        .arg(Arg::new("path").action(ArgAction::Set).required(true))
        // .arg_new_opts()
        .after_help("Run `substrate help new_chain` for more detailed information.\n")
}

pub fn exec(config: &mut Config, args: &ArgMatches) -> CliResult {
    let path = args.get_one::<String>("path");
    // let name = args.get_one::<String>("name");

    let opts = NewOptions {
        path: path.map(|p| config.cwd().join(p)).unwrap(),
        // name: name.map(|s| s.to_string()),
        name: None,
        template: todo!(),
    };
    ops::new_chain(&opts, config)?;
    let path = path.unwrap();
    // let package_name = if let Some(name) = name { name } else { path };
    let package_name = path;
    // config
    //     .shell()
    //     .status("Created", format!("`{}` package", package_name))?;
    Ok(())
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = GlobalContext)]
#[interactive_clap(output_context = NewChainContext)]
pub struct NewChain {
    #[interactive_clap(value_enum)]
    #[interactive_clap(skip_default_input_arg)]
    /// What is the source of the pallet you'd like to install?
    template: NodeTemplate,
    /// Enter the path for your new chain project (either abosulte or relative to the current directory):
    path: String,
    #[interactive_clap(named_arg)]
    /// Name your chain
    name: InputName,
}

#[derive(Debug, Clone)]
pub struct NewChainContext {
    global_context: Config,
    template: NodeTemplate,
    path: String,
}

// TODO: Use ops::substrate_new::NodeTemplate and implement required traits instead of defining a new enum
#[derive(Debug, Clone, EnumDiscriminants, clap::ValueEnum)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum NodeTemplate {
    #[strum_discriminants(strum(
        message = "substrate            - Create a new chain from Substrate's bare-bones template"
    ))]
    Substrate,
    #[strum_discriminants(strum(
        message = "cumulus              - Create a new Cumulus-based parachain (Polkadot, Kusama, Rococo, etc.)"
    ))]
    Cumulus,
    #[strum_discriminants(strum(
        message = "frontier             - Create a new Frontier-based chain (Ethereum compatible)"
    ))]
    Frontier,
    #[strum_discriminants(strum(
        message = "canvas               - Create a new Canvas-based chain"
    ))]
    Canvas,
}

impl interactive_clap::ToCli for NodeTemplate {
    type CliVariant = NodeTemplate;
}
impl std::str::FromStr for NodeTemplate {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "substrate" => Ok(Self::Substrate),
            "cumulus" => Ok(Self::Cumulus),
            "frontier" => Ok(Self::Frontier),
            "canvas" => Ok(Self::Canvas),
            _ => Err("NodeTemplate: incorrect value entered".to_string()),
        }
    }
}
impl std::fmt::Display for NodeTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Substrate => write!(f, "substrate"),
            Self::Cumulus => write!(f, "cumulus"),
            Self::Frontier => write!(f, "frontier"),
            Self::Canvas => write!(f, "canvas"),
        }
    }
}
impl std::fmt::Display for NodeTemplateDiscriminants {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let msg = self.get_message().unwrap();
        write!(f, "{msg}")
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = NewChainContext)]
#[interactive_clap(output_context = InputNameContext)]
pub struct InputName {
    #[interactive_clap(skip_default_input_arg)]
    /// How would you like to name your chain?
    name: String,
}

impl NewChain {
    fn input_template(_context: &GlobalContext) -> color_eyre::eyre::Result<Option<NodeTemplate>> {
        let variants = NodeTemplateDiscriminants::iter().collect::<Vec<_>>();
        let selected = Select::new(
            "Choose a template to generate a new chain from:",
            variants,
        )
        .prompt()?;
        match selected {
            NodeTemplateDiscriminants::Substrate => Ok(Some(NodeTemplate::Substrate)),
            NodeTemplateDiscriminants::Cumulus => Ok(Some(NodeTemplate::Cumulus)),
            NodeTemplateDiscriminants::Frontier => Ok(Some(NodeTemplate::Frontier)),
            NodeTemplateDiscriminants::Canvas => Ok(Some(NodeTemplate::Canvas)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InputNameContext;

impl NewChainContext {
    pub fn from_previous_context(
        previous_context: GlobalContext,
        scope: &<NewChain as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.config,
            template: scope.template.clone(),
            path: scope.path.clone(),
        })
    }
}

impl InputNameContext {
    pub fn from_previous_context(
        previous_context: NewChainContext,
        scope: &<InputName as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let path = Path::new(&previous_context.path);
        let name = scope.name.clone();
        let template = match previous_context.template {
            NodeTemplate::Substrate => ops::substrate_new::Template::Substrate,
            NodeTemplate::Cumulus => ops::substrate_new::Template::Cumulus,
            NodeTemplate::Frontier => ops::substrate_new::Template::Frontier,
            NodeTemplate::Canvas => ops::substrate_new::Template::Canvas,
        };

        let path = normalize_paths(previous_context.global_context.cwd(), path)?;

        println!("Path: {}", path.display());

        let opts = NewOptions {
            template,
            name: Some(name.clone()),
            path,
        };

        if let Err(e) = ops::new_chain(&opts, &previous_context.global_context) {
            return Err(color_eyre::eyre::eyre!(e));
        }

        Ok(Self)
    }
}

impl InputName {
    fn input_name(context: &NewChainContext) -> color_eyre::eyre::Result<Option<String>> {
        let path_file_name = context
            .global_context
            .cwd()
            .join(&context.path)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let input_name = Text::new("How would you like to name your chain?")
            .with_placeholder(&path_file_name)
            .with_default(&path_file_name)
            .prompt()?;
        let name = input_name.parse()?;
        Ok(Some(name))
    }
}
