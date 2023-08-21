use std::path::PathBuf;

use serde_derive::Deserialize;

use crate::{core::manifest::Manifest, util::SubstrateResult};

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct TemplateConfig {
    pub remote: String,
    pub branch: String,
    pub template_path: String,
}

// TODO: Add support for custom templates
pub fn load_template_config(template_name: &str) -> SubstrateResult<TemplateConfig> {
     // Construct the path to the embedded config file
    let template_config_str = match template_name.to_lowercase().as_str() {
        "substrate" => include_str!("chain/substrate.toml"),
        "cumulus" => include_str!("chain/cumulus.toml"),
        "frontier" => include_str!("chain/frontier.toml"),
        "canvas" => include_str!("chain/canvas.toml"),
        // Add more cases for each config file
        _ => anyhow::bail!("Invalid template name".to_string()),
    };

    let template_config = toml_edit::de::from_str::<TemplateConfig>(template_config_str)?;

    Ok(template_config)
}
