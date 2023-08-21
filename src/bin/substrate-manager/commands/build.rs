use substrate_manager::{ops, util::Config};

use super::GlobalContext;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = GlobalContext)]
#[interactive_clap(output_context = BuildContext)]
pub struct Build;

#[derive(Debug, Clone)]
pub struct BuildContext;

impl BuildContext {
    pub fn from_previous_context(
        _previous_context: GlobalContext,
        _scope: &<Build as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        if let Err(e) = ops::build() {
            return Err(color_eyre::eyre::eyre!(e));
        }
        Ok(Self)
    }
}
