use std::process::Command;

use anyhow::Ok;

use crate::util::{Config, SubstrateResult};

pub struct RunOptions {
    pub chain: String,
}

pub fn run(opts: &RunOptions) -> SubstrateResult<()> {
    let mut args = vec!["+nightly", "run", "--release", "--"];
    if opts.chain == "dev" {
        // 'dev' gets special treatment
        args.push("--dev");
    } else {
        args.extend(["--chain", &opts.chain]);
    }
    Command::new("cargo")
        .args(args)
        .status()?;

    Ok(())
}
