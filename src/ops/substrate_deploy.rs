use anyhow::Ok;

use crate::util::SubstrateResult;

// TODO:
// - Ask the user for their desired chain to deploy to and include it in the URL as the `rpc` arg
pub fn deploy() -> SubstrateResult<()> {
    let url = "https://contracts-ui.substrate.io/";
    println!("If your browser doesn't automatically open, please open the following URL in your browser:\n{}\n", url);
    open::that(url)?;

    Ok(())
}
