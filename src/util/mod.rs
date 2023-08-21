use std::path::{Path, PathBuf};

pub use self::config::Config;
pub use self::errors::{CliError, CliResult, SubstrateResult};

pub mod command_prelude;
pub mod config;
pub mod errors;
pub mod restricted_names;

pub fn canonicalize_paths(root_path: &Path, path: &Path) -> color_eyre::eyre::Result<PathBuf> {
    let canonical_parent = if let Some(parent) = path.parent() {
        root_path.join(parent).canonicalize()?
    } else {
        Path::new("").into()
    };

    Ok(canonical_parent.join(path.file_name().unwrap()))
}

pub fn indented_lines(text: &str) -> String {
    text.lines()
        .map(|line| {
            if line.is_empty() {
                String::from("\n")
            } else {
                format!("  {}\n", line)
            }
        })
        .collect()
}

/// Transform a string to PascalCase string
pub fn to_pascal_case(string: &str) -> String {
    let mut chars: Vec<char> = vec![];
    let mut uppercase_next = false;
    for (i, ch) in string.chars().enumerate() {
        if i == 0 {
            chars.extend(ch.to_uppercase())
        } else if ch == '_' || ch == '-' {
            uppercase_next = true;
        } else if uppercase_next {
            chars.extend(ch.to_uppercase());
            uppercase_next = false;
        } else {
            chars.push(ch);
        }
    }

    String::from_iter(chars)
}

/// Transform a string to snake_case string
pub fn to_snake_case(string: &str) -> String {
    let mut buffer = String::with_capacity(string.len() + string.len() / 2);
    let mut prev_was_delimiter = true; // Start with true to handle cases where the input starts with a non-alphanumeric character

    for c in string.chars() {
        if c.is_ascii_alphanumeric() {
            if c.is_uppercase() {
                if !prev_was_delimiter && !buffer.is_empty() {
                    buffer.push('_');
                }
                buffer.push(c.to_ascii_lowercase());
            } else {
                buffer.push(c);
            }
            prev_was_delimiter = false;
        } else if c == ' ' || c == '-' {
            if !prev_was_delimiter && !buffer.is_empty() {
                buffer.push('_');
            }
            prev_was_delimiter = true;
        }
    }

    buffer
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("HelloWorld"), "HelloWorld");
        assert_eq!(to_pascal_case("Hello_World"), "HelloWorld");
        assert_eq!(to_pascal_case("Hello-World"), "HelloWorld");
        assert_eq!(to_pascal_case("helloWorld"), "HelloWorld");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(to_snake_case("Hello_World"), "hello_world");
        assert_eq!(to_snake_case("Hello-World"), "hello_world");
        assert_eq!(to_snake_case("hello-world"), "hello_world");
        assert_eq!(to_snake_case("helloWorld"), "hello_world");
        assert_eq!(to_snake_case("ABc   wOW"), "a_bc_w_o_w");
    }
}
