use std::path::PathBuf;

use color_eyre::eyre::ContextCompat;
use inquire::{validator::Validation, Text};
use strum::{EnumDiscriminants, EnumIter, EnumMessage};
use substrate_manager::{
    core::manifest::Manifest,
    util::{
        config::{ChainInfo, ProjectType},
        CliError, CliResult, Config,
    },
};
use toml_edit::{value, Item};

use self::{
    add_pallet::AddPallet, build::Build, deploy::Deploy, frontend::Frontend, new_chain::NewChain,
    new_contract::NewContract, run::Run, test::Test,
};

pub mod add_pallet;
pub mod build;
pub mod deploy;
pub mod frontend;
pub mod new_chain;
pub mod new_contract;
pub mod run;
pub mod test;

// pub fn builtin() -> Vec<Command> {
//     vec![new_parachain::cli(), dev::cli(), frontend::cli()]
// }

// pub fn builtin_exec(cmd: &str) -> Option<fn(&mut Config, &ArgMatches) -> CliResult> {
//     let f = match cmd {
//         "new" => new_parachain::exec,
//         "dev" => dev::exec,
//         "frontend" => frontend::exec,
//         _ => return None,
//     };
//     Some(f)
// }
//

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = GlobalContext)]
#[interactive_clap(output_context = GlobalContext)]
pub struct ChainCmd {
    #[interactive_clap(subcommand)]
    top_level: Chain,
}

#[derive(Debug, Clone)]
pub struct GlobalContext {
    pub config: Config,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(disable_back)]
#[non_exhaustive]
/// What's your next move? (Select an option below)
pub enum Chain {
    /// Use this to run the substrate chain node
    #[strum_discriminants(strum(message = "run          - ▶️  Start the chain node"))]
    Run(Run),
    /// Add pallets to your chain
    #[strum_discriminants(strum(message = "add          - 📦 Add pallets to your chain"))]
    Add(AddPallet),
    /// Use this to run the frontent application that connects to the chain node
    #[strum_discriminants(strum(
        message = "frontend     - 📡 Launch the frontend interface for your chain"
    ))]
    Frontend(Frontend),
    /// Use this to run the tests for your chain
    #[strum_discriminants(strum(message = "test         - 🧪 Run the tests for your chain"))]
    Test(Test),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = GlobalContext)]
pub struct ContractCmd {
    #[interactive_clap(subcommand)]
    top_level: Contract,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(disable_back)]
#[non_exhaustive]
/// What's your next move? (Select an option below)
pub enum Contract {
    /// Use this to compile the smart contract into optimized WebAssembly bytecode, generate
    /// metadata for it, and bundle both together in a <name>.contract file
    #[strum_discriminants(strum(
        message = "build          - 🏗️ Compile the contract into wasm (.contract file) for deployment"
    ))]
    Build(Build),
    /// Use this to open the Substrate Smart Contract UI to deploy your smart contract
    #[strum_discriminants(strum(
        message = "deploy         - 🚀 Deploy the contract using Substrate Smart Contract UI"
    ))]
    Deploy(Deploy),
    /// Use this to run the tests for your smart contract
    #[strum_discriminants(strum(
        message = "test           - 🧪 Run tests for the smart contract"
    ))]
    Test(Test),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = GlobalContext)]
// #[interactive_clap(output_context = CmdContext)]
pub struct MissingProjectCmd {
    #[interactive_clap(subcommand)]
    top_level: MissingProject,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(disable_back)]
#[non_exhaustive]
/// What would you like to create today?
pub enum MissingProject {
    #[strum_discriminants(strum(
        message = "new-chain           - 🪂 Create a new chain/parachain project"
    ))]
    /// Create a new substrate chain/parachain project
    NewChain(NewChain),
    #[strum_discriminants(strum(
        message = "new-contract        - 🦑 Create a new smart contract project"
    ))]
    /// Create a new substrate smart-contract project
    NewContract(NewContract),
}

impl GlobalContext {
    pub fn from_previous_context(
        previous_context: GlobalContext,
        _scope: &<ChainCmd as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        if let ProjectType::Chain(ChainInfo {
            node_path,
            runtime_path,
            ..
        }) = &previous_context.config.project_type.clone().unwrap()
        {
            if !node_path.join("Cargo.toml").exists() {
                println!("Could not locate your node.");
                let path = Text::new("Where is your node located?")
                    .with_help_message("The path should be relative to the current directory")
                    .with_validator(|p: &str| {
                        if !p.is_empty() && PathBuf::from(p).join("Cargo.toml").exists() {
                            Ok(Validation::Valid)
                        } else {
                            Ok(Validation::Invalid(
                                "The path you entered does not exist or is not a valid node".into(),
                            ))
                        }
                    })
                    .prompt()?;

                let mut manifest = Manifest::new(previous_context.config.cwd().join("Substrate.toml"));
                let mut document = manifest.read_document().unwrap_or_default();
                document["paths"]
                    .or_insert(toml_edit::table())
                    .as_table_mut()
                    .with_context(|| "unknown `paths` type in Substrate.toml")?
                    .insert("node", value(path));
                if let Err(e) = manifest.write_document(document) {
                    return Err(color_eyre::eyre::eyre!(e));
                }
            }

            if !runtime_path.join("Cargo.toml").exists() {
                println!("Could not locate your runtime.");
                let path = Text::new("Where is your runtime located?")
                    .with_help_message("The path should be relative to the current directory")
                    .with_validator(|p: &str| {
                        if !p.is_empty() && PathBuf::from(p).join("Cargo.toml").exists() {
                            Ok(Validation::Valid)
                        } else {
                            Ok(Validation::Invalid(
                                "The path you entered does not exist or is not a valid runtime"
                                    .into(),
                            ))
                        }
                    })
                    .prompt()?;

                let mut manifest = Manifest::new(previous_context.config.cwd().join("Substrate.toml"));
                let mut document = manifest.read_document().unwrap_or_default();
                document["paths"]
                    .or_insert(toml_edit::table())
                    .as_table_mut()
                    .with_context(|| "unknown `paths` type in Substrate.toml")?
                    .insert("runtime", value(path));
                if let Err(e) = manifest.write_document(document) {
                    return Err(color_eyre::eyre::eyre!(e));
                }
            }

            Ok(Self {
                config: Config::default().unwrap(),
            })
        } else {
            color_eyre::eyre::bail!("Incorrect project type")
        }
    }
}

pub fn main<A, B>(global_context: GlobalContext) -> CliResult
where
    A: interactive_clap::ToCli<CliVariant = B>
        + interactive_clap::FromCli<FromCliContext = GlobalContext, FromCliError = color_eyre::Report>,
    B: interactive_clap::ToCliArgs + clap::Parser + From<A>,
{
    let cli = match B::try_parse() {
        Ok(cli) => cli,
        Err(error) => error.exit(),
    };

    println!("🚀 Welcome to Substrate Manager CLI 🚀");
    if let Config {
        cwd: _,
        project_type: Some(project_type),
    } = global_context.config.clone()
    {
        let emoji = match project_type {
            substrate_manager::util::config::ProjectType::Chain(_) => "🪂",
            substrate_manager::util::config::ProjectType::Contract(_) => "🦑",
        };
        println!("{} {}", emoji, project_type);
    } else {
        println!("No projects found in current directory");
    }
    let cli_cmd = match A::from_cli(Some(cli), global_context) {
        interactive_clap::ResultFromCli::Ok(cli_cmd)
        | interactive_clap::ResultFromCli::Cancel(Some(cli_cmd)) => {
            println!(
                "\nHere is your console command if you need to script it or re-run:\n{}",
                shell_words::join(
                    std::iter::once(
                        std::env::args()
                            .next()
                            .unwrap_or_else(|| "./substrate-manager".to_owned())
                    )
                    .chain(cli_cmd.to_cli_args())
                )
            );
            Ok(Some(cli_cmd))
        }
        interactive_clap::ResultFromCli::Cancel(None) => {
            println!("Goodbye!");
            Ok(None)
        }
        interactive_clap::ResultFromCli::Back => {
            unreachable!("TopLevelCommand does not have back option");
        }
        interactive_clap::ResultFromCli::Err(optional_cli_cmd, err) => {
            if let Some(cli_cmd) = optional_cli_cmd {
                println!(
                    "\nHere is your console command if you need to script it or re-run:\n{}",
                    shell_words::join(
                        std::iter::once(
                            std::env::args()
                                .next()
                                .unwrap_or_else(|| "./substrate-manager".to_owned())
                        )
                        .chain(cli_cmd.to_cli_args())
                    )
                )
            }
            Err(CliError::from(err))
        }
    };

    cli_cmd.map(|_| ())
}
