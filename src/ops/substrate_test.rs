use std::process::Command;

use anyhow::Ok;

use crate::util::{Config, SubstrateResult};

pub struct DevOptions {
    pub debug: bool,
}

pub fn test() -> SubstrateResult<()> {
    Command::new("cargo").args(["+nightly", "test"]).status()?;

    Ok(())
}
