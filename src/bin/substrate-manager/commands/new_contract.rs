use std::path::Path;

use inquire::Text;
use substrate_manager::{
    ops::{self, substrate_new::NewOptions},
    util::{normalize_paths, Config},
};

use super::GlobalContext;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = GlobalContext)]
#[interactive_clap(output_context = NewContractContext)]
pub struct NewContract {
    // #[interactive_clap(skip_default_input_arg)]
    /// Enter the path for your new smart contract project (either abosulte or relative to the current directory):
    path: String,
    #[interactive_clap(named_arg)]
    /// Name your smart contract
    name: InputName,
}

#[derive(Debug, Clone)]
pub struct NewContractContext {
    global_context: Config,
    path: String,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = NewContractContext)]
#[interactive_clap(output_context = InputNameContext)]
pub struct InputName {
    #[interactive_clap(skip_default_input_arg)]
    /// How would you like to name your smart contract?
    name: String,
}

#[derive(Debug, Clone)]
pub struct InputNameContext;

impl NewContractContext {
    pub fn from_previous_context(
        previous_context: GlobalContext,
        scope: &<NewContract as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.config,
            path: scope.path.clone(),
        })
    }
}

impl InputNameContext {
    pub fn from_previous_context(
        previous_context: NewContractContext,
        scope: &<InputName as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let path = Path::new(&previous_context.path);
        let name = scope.name.clone();

        let path = normalize_paths(previous_context.global_context.cwd(), path)?;

        let opts = NewOptions {
            path,
            name: Some(name.clone()),
            template: ops::substrate_new::Template::CargoContract,
        };

        if let Err(e) = ops::new_contract(&opts, &previous_context.global_context) {
            return Err(color_eyre::eyre::eyre!(e));
        }

        Ok(Self)
    }
}

impl InputName {
    fn input_name(context: &NewContractContext) -> color_eyre::eyre::Result<Option<String>> {
        let path_file_name =
            normalize_paths(context.global_context.cwd(), Path::new(&context.path))?
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();

        let input_name = Text::new("How would you like to name your smart contract?")
            .with_placeholder(&path_file_name)
            .with_default(&path_file_name)
            .prompt()?;
        let name = input_name.parse()?;
        Ok(Some(name))
    }
}
