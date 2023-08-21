use std::{process::Command, path::{Path, PathBuf}};

use anyhow::Ok;

use crate::util::SubstrateResult;

pub fn clone_substrate_frontend_template(path: &Path) -> SubstrateResult<()> {
    Command::new("git")
        .args([
            "clone",
            "git@github.com:substrate-developer-hub/substrate-front-end-template.git",
            path.to_str().unwrap()
        ])
        .status()?;
    Command::new("yarn")
        .current_dir(path)
        .args(["install"])
        .status()?;
    Ok(())
}

/*
 * TODO:
 * Error handling
*/
pub fn frontend() -> SubstrateResult<()> {
    Command::new("yarn")
        .current_dir("frontend")
        .args(["start"])
        .status()?;

    Ok(())
}
