use substrate_manager::{ops, util::Config};

use super::GlobalContext;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = GlobalContext)]
#[interactive_clap(output_context = DeployContext)]
pub struct Deploy;

#[derive(Debug, Clone)]
pub struct DeployContext;

impl DeployContext {
    pub fn from_previous_context(
        _previous_context: GlobalContext,
        _scope: &<Deploy as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        if let Err(e) = ops::deploy() {
            return Err(color_eyre::eyre::eyre!(e));
        }
        Ok(Self)
    }
}
