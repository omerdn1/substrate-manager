use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Ok;
use regex::Regex;

use crate::{
    core::manifest::Manifest,
    util::{to_pascal_case, to_snake_case, Config, SubstrateResult},
};

pub struct AddOptions {
    /// The name of the project's package
    pub package_name: String,
    /// The path of the project's package
    pub package_path: PathBuf,
    /// The name of the crate
    pub crate_spec: String,

    /// Feature flags to activate
    pub features: Vec<String>,
    /// Whether the default feature should be activated
    pub default_features: Option<bool>,

    /// Source of the crate
    pub source: CrateSource,
}

pub enum CrateSource {
    DefaultRegistry,
    Git(String, String),
    Path(String),
    CustomRegistry(String),
}

pub fn add_pallet_std_to_manifest(
    cwd: &Path,
    runtime_path: &Path,
    crate_spec: &str,
) -> SubstrateResult<()> {
    let feature = format!("{}/std", crate_spec);
    let mut runtime_manifest = Manifest::new(cwd.join(runtime_path).join("Cargo.toml"));
    let mut runtime_document = runtime_manifest.read_document()?;
    // Add pallet/std to features vector
    let feature_array = runtime_document["features"].as_table_mut().unwrap()["std"]
        .as_array_mut()
        .unwrap();
    if !feature_array.iter().any(|f| f.as_str().unwrap() == feature) {
        feature_array.push::<String>(feature);
    }

    runtime_manifest.write_document(runtime_document)?;

    Ok(())
}

// Inspired by parity's substrate-deps: https://github.com/paritytech/substrate-deps/blob/master/src/runtime.rs#L11
pub fn add_pallet_to_runtime(
    cwd: &Path,
    runtime_path: &Path,
    crate_spec: &str,
) -> SubstrateResult<Option<usize>> {
    let runtime_lib_path = cwd.join(runtime_path).join("src/lib.rs");
    let mod_name = to_snake_case(crate_spec);

    let pallet_trait_existing = Regex::new(
        format!(
            r"(?xm)
                ^impl\s+{}::Config\s+for\s+Runtime\s+\{{
                    [^\}}]+
                \}}
        ",
            mod_name
        )
        .as_ref(),
    )?;

    let construct_runtime = Regex::new(
        r"construct_runtime!\(\s*(?P<visibility>pub\s+)?(?P<variant>enum|struct)\s+Runtime\s*(where[^\{]+)?\{(?P<pallets>[\s\S]+)\}\s*\);",
    )?;

    let mut pallet_trait_impl = format!("impl {}::Config for Runtime {{ \n", mod_name);
    pallet_trait_impl.push_str(&format!("	/* {} Trait config goes here */ \n", mod_name));
    pallet_trait_impl.push('}');

    let pallet_config = format!(
        r"
        {}: {},",
        to_pascal_case(&mod_name),
        mod_name
    );

    let original = fs::read_to_string(&runtime_lib_path)?;
    let mut buffer = original.clone();
    let mut line_number: Option<usize> = None;
    if pallet_trait_existing.is_match(&original) {
        buffer = pallet_trait_existing
            .replace(&original, |caps: &regex::Captures| {
                line_number = Some(original[..caps.get(0).unwrap().start()].lines().count() + 1);
                pallet_trait_impl.to_owned()
            })
            .to_string();
    } else {
        let mat = construct_runtime
            .find(&original)
            .ok_or_else(|| anyhow::anyhow!("couldn't find construct_runtime call"))?;
        line_number = Some(original[..mat.start()].lines().count() + 1);
        buffer.insert_str(mat.start(), format!("{}\n\n", pallet_trait_impl).as_str());
    };

    let modified = buffer.clone();
    let caps = construct_runtime
        .captures(&modified)
        .ok_or_else(|| anyhow::anyhow!("couldn't find construct_runtime call"))?;
    let pallets = caps.name("pallets").ok_or_else(|| {
        anyhow::anyhow!("couldn't find runtime pallets config inside construct_runtime",)
    })?;

    let existing_pallets = modified.get(pallets.start()..pallets.end()).unwrap();
    let line_number_with_mod_name = existing_pallets
        .lines()
        .position(|line| line.contains(&mod_name));

    if let Some(line_number) = line_number_with_mod_name {
        let line_to_replace = existing_pallets.lines().nth(line_number).unwrap();
        let new_existing_pallets = existing_pallets.replace(line_to_replace, &pallet_config[1..]);

        let new_buffer = modified.replacen(existing_pallets, &new_existing_pallets, 1);
        fs::write(runtime_lib_path, new_buffer)?;
    } else {
        // Insert the pallet_config at the end of pallets
        buffer.insert_str(pallets.end() - 2, &pallet_config);
        fs::write(runtime_lib_path, buffer)?;
    }

    Ok(line_number)
}

// TODO:
// - Make sure the crate is a valid pallet
// - Try to implement pallet's `Config` trait for runtime by scraping docs to try to find the default implementation
pub fn add_pallet(opts: &AddOptions, config: &Config) -> SubstrateResult<()> {
    let crate_source_arg = match &opts.source {
        CrateSource::DefaultRegistry => vec![],
        CrateSource::Git(url, branch) => {
            let mut args = vec!["--git", url];
            if !branch.is_empty() {
                args.extend(["--branch", branch]);
            }
            args
        },
        CrateSource::Path(path) => vec!["--path", path],
        CrateSource::CustomRegistry(registry) => vec!["--registry", registry],
    };

    let status = Command::new("cargo")
        .arg("add")
        .arg("-p")
        .arg(&opts.package_name)
        .arg(&opts.crate_spec)
        .arg("--features")
        .arg(&opts.features.join(","))
        .arg("--no-default-features")
        .args(crate_source_arg)
        .status()?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "Failed to install pallet: `{}`",
            opts.crate_spec,
        ));
    }

    add_pallet_std_to_manifest(config.cwd(), &opts.package_path, &opts.crate_spec)?;

    let trait_line_number =
        add_pallet_to_runtime(config.cwd(), &opts.package_path, &opts.crate_spec)?;
    println!(
        "\nPallet `{}` has been successfully added to the runtime!",
        opts.crate_spec
    );
    if let Some(line_number) = trait_line_number {
        println!(
            "Don't forget to implement the `Config` trait in `runtime/src/lib.rs`, line: {}",
            line_number
        );
    }

    Ok(())
}
