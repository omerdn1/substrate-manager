use std::process::Command;

use crate::util::SubstrateResult;

pub fn build() -> SubstrateResult<()> {
    Command::new("cargo-contract")
        .args(["contract", "build"])
        .status()?;

    Ok(())
}
