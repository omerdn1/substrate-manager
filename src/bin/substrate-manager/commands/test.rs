use substrate_manager::ops;

use super::GlobalContext;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = GlobalContext)]
#[interactive_clap(output_context = TestContext)]
pub struct Test;

#[derive(Debug, Clone)]
pub struct TestContext;

impl TestContext {
    pub fn from_previous_context(
        _previous_context: GlobalContext,
        _scope: &<Test as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        if let Err(e) = ops::test() {
            return Err(color_eyre::eyre::eyre!(e));
        }
        Ok(Self)
    }
}
