use anyhow::Context as _;
use core::fmt;
use inquire::Confirm;
use std::{
    env,
    fmt::Debug,
    path::{Path, PathBuf},
    str::FromStr,
};
use toml_edit::{value, Document, Item, Table, Value};

use crate::core::manifest::Manifest;

use super::SubstrateResult;

fn find_field_recursive<'a>(table: &'a Table, field_path: &str) -> Option<&'a Item> {
    let mut current = table;

    for field in field_path.split('.') {
        if let Some(value) = current.get(field) {
            if let Some(next_table) = value.as_table() {
                current = next_table;
            } else {
                return Some(value);
            }
        } else {
            return None;
        }
    }

    None
}

fn from_manifest_or_default<T>(manifest: Option<&Document>, field_path: &str, default: &str) -> T
where
    T: FromStr,
    T::Err: Debug,
{
    let option = match manifest.map(|doc| doc.as_table()) {
        Some(table) => find_field_recursive(table, field_path)
            .and_then(|v| v.as_str())
            .unwrap_or(default),
        None => default,
    };

    option
        .parse::<T>()
        .unwrap_or_else(|_| panic!("Could not parse configuration option: {}", field_path))
}

#[derive(Debug, Clone)]
pub struct Config {
    // shell: RefCell<Shell>,
    pub cwd: PathBuf,
    pub project_type: Option<ProjectType>,
}

// Each chain has a node name and a runtime name
#[derive(Debug, Clone)]
pub struct ChainInfo {
    pub node_path: PathBuf,
    pub node_name: Option<String>,
    pub runtime_path: PathBuf,
    pub runtime_name: Option<String>,
    pub frontend_path: PathBuf,
}

// Each contract has a name
#[derive(Debug, Clone)]
pub struct ContractInfo {
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum ProjectType {
    Chain(ChainInfo),
    Contract(ContractInfo),
}

impl fmt::Display for ProjectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Chain(info) => {
                let node_name = info.node_name.as_deref().unwrap_or("unknown");
                let runtime_name = info.runtime_name.as_deref().unwrap_or("unknown");
                write!(f, "Chain: node - {}, runtime - {}", node_name, runtime_name)
            }
            Self::Contract(info) => write!(f, "Contract: {}", info.name),
        }
    }
}

pub fn get_package_name(path: &Path) -> SubstrateResult<Option<String>> {
    // Get node name from manifest file
    let name = if path.join("Cargo.toml").exists() {
        let manifest = Manifest::new(path.join("Cargo.toml")).read_document()?;
        Some(
            manifest["package"]
                .get("name")
                .with_context(|| "no name found in manifest")?
                .as_str()
                .with_context(|| "unknown name type in manifest")?
                .to_string(),
        )
    } else {
        None
    };

    Ok(name)
}

// TODO: Move `inquire` related code to the bin module
fn deduce_project_type_for_wd(
    cwd: &Path,
    node_path: &Path,
    runtime_path: &Path,
    frontend_path: &Path,
) -> SubstrateResult<Option<ProjectType>> {
    if cwd.join(node_path).exists() || cwd.join(runtime_path).exists() {
        if Confirm::new("Found a potential chain project in the current directory. Do you want to continue? (y/n)").prompt()? {
                let mut manifest = Manifest::new(cwd.join("Substrate.toml"));
                let mut document = Document::new();
                document.insert("type", value("chain"));
                manifest.write_document(document)?;
            } else {
                return Ok(None);
            }

        let node_name = get_package_name(&cwd.join(node_path))?;
        let runtime_name = get_package_name(&cwd.join(runtime_path))?;

        return Ok(Some(ProjectType::Chain(ChainInfo {
            node_path: PathBuf::from(node_path),
            node_name,
            runtime_path: PathBuf::from(runtime_path),
            runtime_name,
            frontend_path: PathBuf::from(frontend_path),
        })));
    } else if cwd.join("lib.rs").exists() {
        if Confirm::new("Found a potential smart contract project in the current directory. Do you want to continue? (y/n)").prompt()? {
                let mut manifest = Manifest::new(cwd.join("Substrate.toml"));
                let mut document = Document::new();
                document.insert("type", value("contract"));
                manifest.write_document(document)?;
            } else {
                return Ok(None);
            }

        // Get contract name from manifest file
        let contract_manifest = Manifest::new(cwd.join("Cargo.toml")).read_document()?;
        let contract_name = contract_manifest["package"]
            .get("name")
            .with_context(|| "no name found in contract manifest")?
            .as_str()
            .with_context(|| "unknown name type in contract manifest")?
            .to_string();

        return Ok(Some(ProjectType::Contract(ContractInfo {
            name: contract_name,
        })));
    }

    Ok(None)
}

impl Config {
    // This is typically used for tests.
    // Normally we'd used the default method to build a config.
    pub fn new(cwd: PathBuf, project_type: Option<ProjectType>) -> Self {
        Self {
            // shell: RefCell::new(shell),
            cwd,
            project_type,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn default() -> SubstrateResult<Self> {
        // let shell = Shell::new();
        let cwd = env::current_dir()
            .with_context(|| "couldn't get the current directory of the process")?;

        let manifest = Manifest::new(cwd.join("Substrate.toml"))
            .read_document()
            .ok();

        let node_path =
            from_manifest_or_default::<PathBuf>(manifest.as_ref(), "paths.node", "node");
        let runtime_path =
            from_manifest_or_default::<PathBuf>(manifest.as_ref(), "paths.runtime", "runtime");
        let frontend_path =
            from_manifest_or_default::<PathBuf>(manifest.as_ref(), "paths.frontend", "frontend");

        let contract_path =
            from_manifest_or_default::<PathBuf>(manifest.as_ref(), "paths.contract", "");

        let project_type = match manifest.as_ref() {
            Some(doc) => match doc["type"].as_str() {
                Some("chain") => {
                    let node_name = get_package_name(&cwd.join(&node_path))?;
                    let runtime_name = get_package_name(&cwd.join(&runtime_path))?;

                    Some(ProjectType::Chain(ChainInfo {
                        node_path,
                        node_name,
                        runtime_path,
                        runtime_name,
                        frontend_path,
                    }))
                }
                Some("contract") => {
                    let name = get_package_name(&cwd.join(contract_path))?;
                    Some(ProjectType::Contract(ContractInfo {
                        name: name.unwrap(),
                    }))
                }
                _ => anyhow::bail!("Incorrect project type in Substrate.toml.\nSupported types: \"chain\" and \"contract\"."),
            },
            None => deduce_project_type_for_wd(&cwd, &node_path, &runtime_path, &frontend_path)?,
        };

        Ok(Self::new(cwd, project_type))
    }

    /// Gets a reference to the shell, e.g., for writing error messages.
    // pub fn shell(&self) -> RefMut<'_, Shell> {
    //     self.shell.borrow_mut()
    // }

    /// The current working directory.
    pub fn cwd(&self) -> &Path {
        &self.cwd
    }

    /// Retrieves the project type.
    pub fn project_type(&self) -> &Option<ProjectType> {
        &self.project_type
    }
}
