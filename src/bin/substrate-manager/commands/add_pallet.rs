use inquire::{Select, Text};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};
use substrate_manager::{
    ops::{
        self,
        substrate_add::{AddOptions, CrateSource},
    },
    util::config::{ChainInfo, ProjectType},
};

use super::GlobalContext;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = GlobalContext)]
#[interactive_clap(output_context = AddPalletContext)]
pub struct AddPallet {
    /// What is the name of the pallet you'd like to install?
    name: String,
    /// Space or comma separated list of features to activate (leave empty for default):
    features: String,
    #[interactive_clap(value_enum)]
    #[interactive_clap(skip_default_input_arg)]
    /// What is the source of the pallet you'd like to install?
    source: PalletSource,
}

#[derive(Debug, Clone)]
pub struct AddPalletContext;

#[derive(Debug, Clone, EnumDiscriminants, clap::ValueEnum)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum PalletSource {
    #[strum_discriminants(strum(
        message = "default-registry     - Install pallet from the default registry (crates.io)"
    ))]
    DefaultRegistry,
    #[strum_discriminants(strum(
        message = "git                  - Install pallet from a git repository"
    ))]
    Git,
    #[strum_discriminants(strum(
        message = "path                 - Install pallet from a local path"
    ))]
    Path,
    #[strum_discriminants(strum(
        message = "custom-registry      - Install pallet from a custom registry"
    ))]
    CustomRegistry,
}

impl interactive_clap::ToCli for PalletSource {
    type CliVariant = PalletSource;
}
impl std::str::FromStr for PalletSource {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "default-registry" => Ok(Self::DefaultRegistry),
            "git" => Ok(Self::Git),
            "path" => Ok(Self::Path),
            "custom-registry" => Ok(Self::CustomRegistry),
            _ => Err("PalletSource: incorrect value entered".to_string()),
        }
    }
}
impl std::fmt::Display for PalletSource {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::DefaultRegistry => write!(f, "default-registry"),
            Self::Git => write!(f, "git"),
            Self::Path => write!(f, "path"),
            Self::CustomRegistry => write!(f, "custom-registry"),
        }
    }
}
impl std::fmt::Display for PalletSourceDiscriminants {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let msg = self.get_message().unwrap();
        write!(f, "{msg}")
    }
}

impl AddPallet {
    fn input_source(_context: &GlobalContext) -> color_eyre::eyre::Result<Option<PalletSource>> {
        let variants = PalletSourceDiscriminants::iter().collect::<Vec<_>>();
        let selected = Select::new(
            "What is the source of the pallet you'd like to install?",
            variants,
        )
        .prompt()?;
        match selected {
            PalletSourceDiscriminants::DefaultRegistry => Ok(Some(PalletSource::DefaultRegistry)),
            PalletSourceDiscriminants::Git => Ok(Some(PalletSource::Git)),
            PalletSourceDiscriminants::Path => Ok(Some(PalletSource::Path)),
            PalletSourceDiscriminants::CustomRegistry => Ok(Some(PalletSource::CustomRegistry)),
        }
    }
}

impl AddPalletContext {
    pub fn from_previous_context(
        previous_context: GlobalContext,
        scope: &<AddPallet as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let features = parse_features(&scope.features)
            .map(|f| f.to_string())
            .collect::<Vec<String>>();

        let source = match scope.source {
            PalletSource::DefaultRegistry => CrateSource::DefaultRegistry,
            PalletSource::Git => {
                let input_git_repository = Text::new("What is the git repository URL?")
                    .with_default("https://github.com/paritytech/substrate.git")
                    .with_placeholder("https://github.com/paritytech/substrate.git")
                    .prompt()?;
                let git_repository = input_git_repository.parse()?;

                let (default, placeholder) = if git_repository == "https://github.com/paritytech/substrate.git" {
                    ("polkadot-v1.0.0", "polkadot-v1.0.0")
                } else {
                    ("", "master")
                };

                let input_git_branch = Text::new("What is the git branch?")
                    .with_default(default)
                    .with_placeholder(placeholder)
                    .prompt()?;
                let git_branch = input_git_branch.parse()?;

                CrateSource::Git(git_repository, git_branch)
            }
            PalletSource::Path => {
                let input_path = Text::new("What is the local path?").prompt()?;
                let path = input_path.parse()?;
                CrateSource::Path(path)
            }
            PalletSource::CustomRegistry => {
                let input_custom_registry = Text::new("What is the custom registry?").prompt()?;
                let custom_registry = input_custom_registry.parse()?;
                CrateSource::CustomRegistry(custom_registry)
            }
        };

        if let ProjectType::Chain(ChainInfo {
            runtime_name,
            runtime_path,
            ..
        }) = &previous_context.config.project_type.clone().unwrap()
        {
            let opts = AddOptions {
                package_name: runtime_name.clone().unwrap().clone(),
                package_path: runtime_path.clone(),
                crate_spec: scope.name.clone(),
                features,
                default_features: Some(false),
                source,
            };

            if let Err(e) = ops::add_pallet(&opts, &previous_context.config) {
                return Err(color_eyre::eyre::eyre!(e));
            }

            Ok(Self)
        } else {
            color_eyre::eyre::bail!("Incorrect project type")
        }
    }
}

/// Split feature flag list
fn parse_features(feature: &str) -> impl Iterator<Item = &str> {
    // Not re-using `CliFeatures` because it uses a BTreeSet and loses user's ordering
    feature
        .split_whitespace()
        .flat_map(|s| s.split(','))
        .filter(|s| !s.is_empty())
}
