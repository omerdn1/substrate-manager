use anyhow::Context as _;
use core::slice;
use std::ffi::OsStr;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use strum::Display;
use toml_edit::value;
use toml_edit::Array;
use toml_edit::Document;
use toml_edit::Item;
use toml_edit::Table;

use crate::core::manifest::Manifest;
use crate::templates::load_template_config;
use crate::util::config::get_package_name;
use crate::util::config::Config;
use crate::util::restricted_names;
use crate::util::to_snake_case;
use crate::util::SubstrateResult;

#[derive(Debug, Display)]
pub enum Template {
    // Chain templates
    Substrate,
    Cumulus,
    Frontier,
    Canvas,
    // Contract templates
    CargoContract,
    // Path to custom template
    Custom(String),
}

#[derive(Debug)]
pub struct NewOptions {
    pub template: Template,
    /// Absolute path to the directory for the new package
    pub path: PathBuf,
    pub name: Option<String>,
}

// TODO: Add digestible error message on failure
pub fn validate_rust_installation() -> SubstrateResult<()> {
    let info = String::from_utf8_lossy(
        &Command::new("rustup")
            // Workaround for override caused by RUSTUP_TOOLCHAIN
            .args(vec!["run", "nightly", "rustup", "show"])
            .output()?
            .stdout,
    )
    .to_string();

    if !info.contains("wasm32-unknown-unknown") || !info.contains("nightly") {
        println!("\nRust nightly toolchain is not installed. Installing...\n");
        // Follows installation steps in: https://docs.substrate.io/install/macos/
        // @TODO: Make sure same steps work for other operating systems.
        Command::new("rustup")
            .args(vec!["default", "stable"])
            .status()?;
        Command::new("rustup").args(vec!["update"]).status()?;
        Command::new("rustup")
            .args(vec!["update", "nightly"])
            .status()?;
        Command::new("rustup")
            .args(vec![
                "target",
                "add",
                "wasm32-unknown-unknown",
                "--toolchain",
                "nightly",
            ])
            .status()?;
    }

    Ok(())
}

fn get_name(opts: &NewOptions) -> SubstrateResult<&str> {
    if let Some(ref name) = opts.name {
        return Ok(name);
    }

    let path = &opts.path;
    let file_name = path.file_name().ok_or_else(|| {
        anyhow::format_err!(
            "cannot auto-detect package name from path {:?} ; use --name to override",
            path.as_os_str()
        )
    })?;

    file_name.to_str().ok_or_else(|| {
        anyhow::format_err!(
            "cannot create package with a non-unicode name: {:?}",
            file_name
        )
    })
}

fn get_parent(opts: &NewOptions) -> SubstrateResult<&str> {
    let path = &opts.path;
    let parent = path.parent().ok_or_else(|| {
        anyhow::format_err!(
            "cannot auto-detect package parent directory from path {:?}",
            path.as_os_str()
        )
    })?;

    parent.to_str().ok_or_else(|| {
        anyhow::format_err!(
            "cannot create package with a non-unicode name: {:?}",
            parent
        )
    })
}

/// Validates that the path contains valid PATH env characters.
fn validate_path(path: &Path) -> SubstrateResult<()> {
    if cargo_util::paths::join_paths(slice::from_ref(&OsStr::new(path)), "").is_err() {
        let path = path.to_string_lossy();
        anyhow::bail!(
            "the path `{path}` contains invalid PATH characters (usually `:`, `;`, or `\"`)\n\
            It is recommended to use a different name to avoid problems."
        );
    }
    Ok(())
}

fn validate_name(name: &str, show_name_help: bool) -> SubstrateResult<()> {
    // If --name is already used to override, no point in suggesting it
    // again as a fix.
    let name_help = if show_name_help {
        "\nIf you need a package name to not match the directory name, consider using --name flag."
    } else {
        ""
    };
    let bin_help = String::from(name_help);

    restricted_names::validate_package_name(name, "package name", &bin_help)?;

    if restricted_names::is_keyword(name) {
        anyhow::bail!(
            "the name `{}` cannot be used as a package name, it is a Rust keyword{}",
            name,
            bin_help
        );
    }
    if restricted_names::is_conflicting_artifact_name(name) {
        anyhow::bail!(
            "the name `{}` cannot be used as a package name, \
                it conflicts with cargo's build directory names{}",
            name,
            name_help
        );
    }
    if name == "test" {
        anyhow::bail!(
            "the name `test` cannot be used as a package name, \
            it conflicts with Rust's built-in test library{}",
            bin_help
        );
    }
    if ["core", "std", "alloc", "proc_macro", "proc-macro"].contains(&name) {
        let warning = format!(
            "the name `{}` is part of Rust's standard library\n\
            It is recommended to use a different name to avoid problems.{}",
            name, bin_help
        );

        println!("{}", warning);
    }
    if restricted_names::is_windows_reserved(name) {
        if cfg!(windows) {
            anyhow::bail!(
                "cannot use name `{}`, it is a reserved Windows filename{}",
                name,
                name_help
            );
        } else {
            let warning = format!(
                "the name `{}` is a reserved Windows filename\n\
                This package will not work on Windows platforms.",
                name
            );
            println!("{}", warning);
        }
    }
    if restricted_names::is_non_ascii_name(name) {
        let warning = format!(
            "the name `{}` contains non-ASCII characters\n\
            Non-ASCII crate names are not supported by Rust.",
            name
        );
        println!("{}", warning);
    }

    Ok(())
}

fn print_start_hacking_message(cwd: &Path, path: &Path) {
    println!("\nStart hacking by typing:\n");
    if let Ok(relative_path) = path.strip_prefix(cwd) {
        println!("cd {}", relative_path.display());
    } else {
        println!("cd {}", path.display());
    }
    println!("substrate-manager");
}

pub fn new_chain(opts: &NewOptions, config: &Config) -> SubstrateResult<()> {
    let path = &opts.path;
    if path.exists() {
        anyhow::bail!(
            "destination `{}` already exists\n\n\
             Use `substrate init` to initialize the directory",
            path.display()
        )
    }

    validate_path(path)?;

    let name = get_name(opts)?;
    validate_name(name, opts.name.is_none())?;

    validate_rust_installation()?;

    println!("Creating new chain...\n");

    generate_node_template(&opts.template, &path)?;

    mk_chain(opts, name)?;

    println!("\nCreated chain `{}`!", name);
    print_start_hacking_message(config.cwd(), path);

    Ok(())
}

pub fn new_contract(opts: &NewOptions, config: &Config) -> SubstrateResult<()> {
    let path = &opts.path;
    if path.exists() {
        anyhow::bail!(
            "destination `{}` already exists\n\n\
             Use `substrate init` to initialize the directory",
            path.display()
        )
    }

    validate_path(path)?;

    let name = get_name(opts)?;
    validate_name(name, opts.name.is_none())?;

    validate_rust_installation()?;
    validate_cargo_contract_installation()?;

    let parent = get_parent(opts)?;

    println!("Creating new contract...");
    create_smart_contract(name, parent)?;
    mk_contract(opts, name)?;

    print_start_hacking_message(config.cwd(), path);

    Ok(())
}

/// Gets the latest commit id (SHA1) of the repository given by `path`.
fn get_git_commit_id(path: &Path) -> String {
    let commit_id_output = Command::new("git")
        .current_dir(path)
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("git rev-parse failed")
        .stdout;

    let commit_id = String::from_utf8_lossy(&commit_id_output);

    let commit_id = commit_id.trim().to_string();
    println!("Commit id: {}", commit_id);
    commit_id
}

/// Find all `Cargo.toml` files in the given path.
fn find_cargo_tomls(path: &Path) -> Vec<PathBuf> {
    let path = format!("{}/**/Cargo.toml", path.display());

    let glob = glob::glob(&path).expect("Generates globbing pattern");

    let mut result = Vec::new();
    glob.into_iter().for_each(|file| match file {
        Ok(file) => result.push(file),
        Err(e) => println!("{:?}", e),
    });

    if result.is_empty() {
        panic!("Did not find any `Cargo.toml` files.");
    }

    result
}

/// Find all `.rs` files in the given path.
fn find_rust_files(path: &Path) -> Vec<PathBuf> {
    let path = format!("{}/**/*.rs", path.display());

    let glob = glob::glob(&path).expect("Generates globbing pattern");

    let mut result = Vec::new();
    glob.into_iter().for_each(|file| match file {
        Ok(file) => result.push(file),
        Err(e) => println!("{:?}", e),
    });

    if result.is_empty() {
        panic!("Did not find any `.rs` files.");
    }

    result
}

/// Process and replace dependencies in the provided table.
/// Replaces 'path' dependencies with 'git' dependencies if the path does not exist.
fn process_and_replace_dependencies(
    dependencies: &mut Table,
    remote: &str,
    commit_id: &str,
    cargo_toml_path: &Path,
) {
    for (_, dep_value) in dependencies.iter_mut() {
        if let Some(dep_table) = dep_value.as_inline_table_mut() {
            if let Some(path_value) = dep_table.get("path").and_then(|p| p.as_str()) {
                let full_path = cargo_toml_path.join(path_value);
                if !full_path.exists() {
                    dep_table.remove("path");
                    dep_table.insert("git", remote.into());
                    dep_table.insert("rev", commit_id.into());
                }
            }
            *dep_value = value(dep_table.clone());
        }
    }
}

/// Replaces all non-existent remote path dependencie in Cargo.toml files with a git dependency.
fn replace_path_dependencies_with_git(
    cargo_toml_path: &Path,
    remote: &str,
    commit_id: &str,
    cargo_toml: &mut Document,
) {
    let mut cargo_toml_path = cargo_toml_path.to_path_buf();
    // remove `Cargo.toml`
    cargo_toml_path.pop();

    // Process regular dependency tables
    for &table in &["dependencies", "build-dependencies", "dev-dependencies"] {
        if let Some(dependencies) = cargo_toml[table].as_table_mut() {
            process_and_replace_dependencies(dependencies, remote, commit_id, &cargo_toml_path);
        }
    }

    // Process workspace dependency table
    if let Some(workspace_deps) = cargo_toml
        .get_mut("workspace")
        .and_then(|w| w["dependencies"].as_table_mut())
    {
        process_and_replace_dependencies(workspace_deps, remote, commit_id, &cargo_toml_path);
    }
}

/// Update the top level (workspace) `Cargo.toml` file.
///
/// - Adds `profile.release` = `panic = unwind`
/// - Adds `workspace` definition
fn update_top_level_cargo_toml(
    cargo_toml: &mut Document,
    workspace_members: Vec<&PathBuf>,
    node_template_generated_folder: &Path,
) {
    let mut panic_unwind = Table::new();
    panic_unwind.insert("panic", value("unwind"));

    let mut profile = Table::new();
    profile.insert("release", Item::Table(panic_unwind));

    cargo_toml.insert("profile", Item::Table(profile));

    let members = workspace_members
        .iter()
        .map(|p| {
            p.strip_prefix(node_template_generated_folder)
                .expect("Workspace member is a child of the node template path!")
                .parent()
                // We get the `Cargo.toml` paths as workspace members, but for the `members` field
                // we just need the path.
                .expect("The given path ends with `Cargo.toml` as file name!")
                .display()
                .to_string()
        })
        .collect::<Array>();

    // let mut members_section = Table::new();
    // members_section.insert("members", value(members));

    // cargo_toml.insert("workspace", Item::Table(members_section));
    cargo_toml
        .as_table_mut()
        .entry("workspace")
        .or_insert(toml_edit::table())
        .as_table_mut()
        .unwrap()
        .insert("members", value(members));
}

pub fn generate_node_template(template: &Template, path: &Path) -> SubstrateResult<()> {
    let template_config = if let Template::Custom(template_config_path) = template {
        load_template_config(template_config_path)?
    } else {
        load_template_config(&template.to_string())?
    };

    Command::new("git")
        .args([
            "clone",
            "--filter=blob:none",
            "--depth",
            "1",
            "--sparse",
            "--branch",
            &template_config.branch,
            &template_config.remote,
            path.as_os_str()
                .to_str()
                .expect("invalid characters in path"),
        ])
        .status()?;

    // Get commit id before we mutate the repository
    let commit_id = get_git_commit_id(path);

    Command::new("git")
        .current_dir(path)
        .args(["sparse-checkout", "add", &template_config.template_path])
        .status()?;

    // Remove .git directory and reinitialize it
    fs::remove_dir_all(path.join(".git"))?;
    Command::new("git")
        .current_dir(path)
        .args(["init"])
        .status()?;

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let entry_path = entry.path();

                if let Some(file_name) = entry_path.file_name().and_then(|n| n.to_str()) {
                    if entry_path.is_file()
                        && !file_name.contains("rustfmt.toml")
                        && !file_name.contains("Cargo")
                    {
                        fs::remove_file(entry_path)?;
                    }
                }
            }
        }
    }

    let local_template_path = path.join(template_config.template_path);

    for entry in fs::read_dir(&local_template_path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let relative_path = path.join(entry_path.file_name().unwrap());
        let dest_path = path.join(relative_path);

        fs::rename(&entry_path, &dest_path)?;
    }

    fs::remove_dir_all(local_template_path)?;

    let top_level_cargo_toml_path = path.join("Cargo.toml");
    let mut cargo_tomls = find_cargo_tomls(path);

    // Check if top level Cargo.toml exists. If not, create one in the destination
    if !cargo_tomls.contains(&top_level_cargo_toml_path) {
        // create the top_level_cargo_toml
        OpenOptions::new()
            .create(true)
            .write(true)
            .open(&top_level_cargo_toml_path)
            .expect("Create root level `Cargo.toml` failed.");

        // push into our data structure
        cargo_tomls.push(PathBuf::from(&top_level_cargo_toml_path));
    }

    cargo_tomls.iter().for_each(|t| {
        let mut cargo_toml = Manifest::new(t.to_path_buf());
        let mut cargo_toml_document = cargo_toml.read_document().expect("Read Cargo.toml failed.");
        // println!("cargo_toml_document: {:?}", cargo_toml_document);
        replace_path_dependencies_with_git(
            t,
            &template_config.remote,
            &commit_id,
            &mut cargo_toml_document,
        );

        // Check if this is the top level `Cargo.toml`, as this requires some special treatments.
        if top_level_cargo_toml_path == t.to_path_buf() {
            println!("Updating top level `Cargo.toml': {}", t.display());
            // All workspace member `Cargo.toml` file paths.
            let workspace_members = cargo_tomls
                .iter()
                .filter(|p| **p != top_level_cargo_toml_path)
                .collect();

            update_top_level_cargo_toml(&mut cargo_toml_document, workspace_members, path);
        }

        cargo_toml
            .write_document(cargo_toml_document)
            .expect("Write Cargo.toml failed.");
    });

    Ok(())
}

pub fn validate_cargo_contract_installation() -> SubstrateResult<()> {
    if which::which("cargo-contract").is_err() {
        // Install cargo-contract
        Command::new("cargo")
            .args(["install", "--force", "--locked", "cargo-contract"])
            .status()?;
    }

    Ok(())
}

pub fn create_smart_contract(name: &str, parent: &str) -> SubstrateResult<()> {
    // Recursively create project directory and all of its parent directories if they are missing
    fs::create_dir_all(parent)?;

    let status = Command::new("cargo-contract")
        .args(["contract", "new", name, "-t", parent])
        .status()?;

    if !status.success() {
        return Err(anyhow::anyhow!("failed to create smart contract"));
    }

    Ok(())
}

fn replace_occurrence_in_file(file_path: &Path, original: &str, new: &str) -> SubstrateResult<()> {
    if file_path.exists() {
        let file = fs::read_to_string(file_path)?;
        let new_file = file.replace(original, new);
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(file_path)?;
        file.write_all(new_file.as_bytes())?;
    }
    Ok(())
}

pub fn mk_chain(opts: &NewOptions, name: &str) -> SubstrateResult<()> {
    let node_path = opts.path.join("node");
    let node_manifest_path = node_path.join("Cargo.toml");
    let runtime_path = opts.path.join("runtime");
    let runtime_manifest_path = runtime_path.join("Cargo.toml");
    let substrate_manifest_path = opts.path.join("Substrate.toml");

    // TODO:
    // Consider changing package version as well
    let original_runtime_package_name =
        get_package_name(&runtime_path)?.expect("Runtime package name exists");
    let original_runtime_package_name_snake = to_snake_case(&original_runtime_package_name);
    let original_node_package_name =
        get_package_name(&node_path)?.expect("Node package version exists");
    let _original_node_package_name_snake = to_snake_case(&original_node_package_name);
    let node_package_name = name.to_string() + "-node";
    let _node_package_name_snake = to_snake_case(&node_package_name);
    let runtime_package_name = name.to_string() + "-runtime";
    let runtime_package_name_snake = to_snake_case(&runtime_package_name);

    let mut node_manifest = Manifest::new(node_manifest_path);
    let mut node_document = node_manifest.read_document()?;

    let mut runtime_manifest = Manifest::new(runtime_manifest_path);
    let mut runtime_document = runtime_manifest.read_document()?;

    let mut substrate_manifest = Manifest::new(substrate_manifest_path);
    let mut substrate_document = Document::new();

    // Replaces all occurences of 'node-template' with node name
    node_document["package"]["name"] = value(&node_package_name);
    if node_document.get("bin").is_some() {
        node_document["bin"][0]["name"] = value(&node_package_name);
    }

    // TODO:
    // Insert edited runtime_package_name into the right position under 'packages', instead of at
    // the bottom:
    // https://docs.rs/toml_edit/latest/toml_edit/struct.Table.html#method.position

    // Replaces all occurences of `node-template-runtime` with runtime name
    let item = node_document["dependencies"]
        .as_table_mut()
        .unwrap()
        .remove_entry(&original_runtime_package_name)
        .unwrap()
        .1;
    node_document["dependencies"][&runtime_package_name] = item;

    // Deal with the scenario where node inherts from daddy Cargo.toml
    let mut root_manifest = Manifest::new(opts.path.join("Cargo.toml"));
    let mut root_document = root_manifest.read_document()?;
    if let Some(workspace_deps) = root_document["workspace"]["dependencies"].as_table_mut() {
        if let Some(mut runtime_dep) = workspace_deps.remove_entry(&original_runtime_package_name) {
            let dep_table = runtime_dep.1.as_inline_table_mut().unwrap();
            dep_table.remove("git");
            dep_table.remove("rev");
            let path = runtime_path.strip_prefix(&opts.path)?;
            dep_table.insert("path", path.to_str().unwrap().into());
            root_document["workspace"]["dependencies"][&runtime_package_name] = runtime_dep.1;
            root_manifest.write_document(root_document)?;
        }
    }

    // Iterate over all features and replace original runtime name with new one
    for feature in node_document["features"].as_table_mut().unwrap().iter_mut() {
        for arr in feature.1.as_array_mut().unwrap().iter_mut() {
            if arr
                .as_str()
                .unwrap()
                .contains(&original_runtime_package_name)
            {
                *arr = arr
                    .as_str()
                    .unwrap()
                    .replace(&original_runtime_package_name, &runtime_package_name)
                    .into();
            }
        }
    }

    runtime_document["package"]["name"] = value(&runtime_package_name);

    let node_rust_files = find_rust_files(&node_path);
    // let runtime_rust_files = find_rust_files(&runtime_path);

    for file in node_rust_files {
        replace_occurrence_in_file(
            &file,
            &original_runtime_package_name_snake,
            &runtime_package_name_snake,
        )?;
    }

    substrate_document.insert("type", value("chain"));

    // Write changes to files
    node_manifest.write_document(node_document)?;
    runtime_manifest.write_document(runtime_document)?;
    substrate_manifest.write_document(substrate_document)?;

    Ok(())
}

pub fn mk_contract(opts: &NewOptions, _name: &str) -> SubstrateResult<()> {
    let substrate_manifest_path = opts.path.join("Substrate.toml");
    let mut substrate_manifest = Manifest::new(substrate_manifest_path);
    let mut substrate_document = Document::new();

    substrate_document.insert("type", value("contract"));
    substrate_manifest.write_document(substrate_document)?;

    Ok(())
}
