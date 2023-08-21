use std::fs::File;
use std::io::Read;

use color_eyre::eyre::Context;
use inquire::Select;
use substrate_manager::{
    ops::{self, substrate_run::RunOptions},
    util::config::{ChainInfo, ProjectType},
};

use super::GlobalContext;

// TODO: compartmentalize this function into smaller functions
fn extract_match_fields(content: &str) -> Vec<String> {
    // Parse the source code using syn to extract match arms and patterns.
    let parsed_code = syn::parse_file(content).expect("Failed to parse code");

    let mut extracted_fields = Vec::new();

    // Traverse the parsed code to find the SubstrateCli trait implementation and the load_spec method.
    for item in parsed_code.items {
        if let syn::Item::Impl(impl_item) = &item {
            if let Some((_, path, ..)) = &impl_item.trait_ {
                if path.segments.last().map(|s| s.ident.to_string())
                    == Some("SubstrateCli".to_string())
                {
                    // Found SubstrateCli trait implementation.
                    // Now traverse through the methods to find load_spec.
                    for item in &impl_item.items {
                        if let syn::ImplItem::Fn(method) = item {
                            if method.sig.ident == "load_spec" {
                                extracted_fields
                                    .extend(extract_fields_from_load_spec(&method.block));
                            }
                        }
                    }
                }
            }
        }
         if let syn::Item::Fn(fn_item) = &item {
            // Check if the function is the load_spec function declared outside the impl block.
            if fn_item.sig.ident == "load_spec" {
                extracted_fields.extend(extract_fields_from_load_spec(&fn_item.block));
            }
        }
    }

    extracted_fields
}

fn extract_fields_from_load_spec(block: &syn::Block) -> Vec<String> {
    let mut extracted_fields = Vec::new();

    for stmt in &block.stmts {
        if let syn::Stmt::Expr(expr, _) = stmt {
            if let syn::Expr::Call(call_expr) = expr {
                for expr in &call_expr.args {
                    if let syn::Expr::Match(match_expr) = expr {
                        for arm in &match_expr.arms {
                            if let Some(fields) = extract_field_names(&arm.pat) {
                                extracted_fields.extend(fields);
                            }
                        }
                    }
                }
            }
        }
    }

    extracted_fields
}

fn extract_field_names(pat: &syn::Pat) -> Option<Vec<String>> {
    match pat {
        syn::Pat::Ident(_) => {
            // TODO: Implement path identifier as argument
            // return Some(vec!["path".to_string()]);
        }
        syn::Pat::Lit(syn::PatLit { lit, .. }) => {
            // 'dev' => vec!['dev']
            if let syn::Lit::Str(lit_str) = lit {
                let value = lit_str.value();
                if !value.is_empty() {
                    return Some(vec![value]);
                }
            }
        }
        syn::Pat::Path(syn::PatPath { path, .. }) => {
            // some::module::CONSTANT => 'constant'
            if let Some(segment) = path.segments.last() {
                return Some(vec![segment.ident.to_string().to_lowercase()]);
            }
        }
        syn::Pat::Or(syn::PatOr { cases, .. }) => {
            // 'dev' | 'test' | some::module::CONSTANT => vec!['dev', 'test', 'constant']
            let fields = cases
                .iter()
                .filter_map(extract_field_names)
                .flatten()
                .collect();
            return Some(fields);
        }
        _ => panic!("Couldn't extract field with unsupported field pattern"),
    }

    None
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = GlobalContext)]
#[interactive_clap(output_context = RunContext)]
pub struct Run {
    /// What is the chain-specification command you want to run?
    #[interactive_clap(skip_default_input_arg)]
    chain: String,
}

impl Run {
    fn input_chain(context: &GlobalContext) -> color_eyre::eyre::Result<Option<String>> {
        if let ProjectType::Chain(ChainInfo { node_path, .. }) =
            &context.config.project_type.clone().unwrap()
        {
            let mut file = File::open(node_path.join("src/command.rs")).with_context(|| "Couldn't access the node's command.rs file.\nMake sure your node path is set correctly in Substrate.toml")?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;

            let variants = extract_match_fields(&content);

            let select_submit = Select::new(
                "What is the chain-specification command you want to run your chain with?",
                variants,
            )
            .prompt();

            match select_submit {
                Ok(value) => Ok(Some(value.clone())),
                Err(
                    inquire::error::InquireError::OperationCanceled
                    | inquire::error::InquireError::OperationInterrupted,
                ) => Ok(None),
                Err(err) => Err(err.into()),
            }
        } else {
            color_eyre::eyre::bail!("Incorrect project type")
        }
    }
}

#[derive(Debug, Clone)]
pub struct RunContext;

impl RunContext {
    pub fn from_previous_context(
        _previous_context: GlobalContext,
        scope: &<Run as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let options = RunOptions {
            chain: scope.chain.clone(),
        };
        if let Err(e) = ops::run(&options) {
            return Err(color_eyre::eyre::eyre!(e));
        }

        Ok(Self)
    }
}
