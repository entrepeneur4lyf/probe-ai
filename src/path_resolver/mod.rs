//! Module for resolving special path formats to filesystem paths.
//!
//! This module provides functionality to resolve special path formats like
//! "go:github.com/user/repo", "js:express", or "rust:serde" to actual filesystem paths.

mod go;
mod javascript;
mod rust;

use std::path::{Path, PathBuf};

pub use go::GoPathResolver;
pub use javascript::JavaScriptPathResolver;
pub use rust::RustPathResolver;

/// A trait for language-specific path resolvers.
///
/// Implementations of this trait provide language-specific logic for resolving
/// package/module names to filesystem paths.
pub trait PathResolver {
    /// The prefix used to identify paths for this resolver (e.g., "go:", "js:", "rust:").
    fn prefix(&self) -> &'static str;

    /// Splits the path string (after the prefix) into the core module/package
    /// identifier and an optional subpath.
    ///
    /// For example, for Go:
    /// - "fmt" -> Ok(("fmt", None))
    /// - "net/http" -> Ok(("net/http", None)) // Stdlib multi-segment
    /// - "github.com/gin-gonic/gin" -> Ok(("github.com/gin-gonic/gin", None))
    /// - "github.com/gin-gonic/gin/examples/basic" -> Ok(("github.com/gin-gonic/gin", Some("examples/basic")))
    ///
    /// For JavaScript:
    /// - "lodash" -> Ok(("lodash", None))
    /// - "lodash/get" -> Ok(("lodash", Some("get")))
    /// - "@types/node" -> Ok(("@types/node", None))
    /// - "@types/node/fs" -> Ok(("@types/node", Some("fs")))
    ///
    /// # Arguments
    /// * `full_path_after_prefix` - The portion of the input path string that comes *after* the resolver's prefix.
    ///
    /// # Returns
    /// * `Ok((String, Option<String>))` - A tuple containing the resolved module name and an optional subpath string.
    /// * `Err(String)` - An error message if the path format is invalid for this resolver.
    fn split_module_and_subpath(
        &self,
        full_path_after_prefix: &str,
    ) -> Result<(String, Option<String>), String>;

    /// Resolves a package/module name to its filesystem location.
    ///
    /// # Arguments
    ///
    /// * `module_name` - The package/module name to resolve (without any subpath)
    ///
    /// # Returns
    ///
    /// * `Ok(PathBuf)` - The filesystem path where the package is located
    /// * `Err(String)` - An error message if resolution fails
    fn resolve(&self, module_name: &str) -> Result<PathBuf, String>;
}

/// Resolves a path that might contain special prefixes to an actual filesystem path.
///
/// Currently supported formats:
/// - "go:github.com/user/repo" - Resolves to the Go module's filesystem path
/// - "js:express" - Resolves to the JavaScript/Node.js package's filesystem path
/// - "rust:serde" - Resolves to the Rust crate's filesystem path
///
/// # Arguments
///
/// * `path` - The path to resolve, which might contain special prefixes
///
/// # Returns
///
/// * `Ok(PathBuf)` - The resolved filesystem path
/// * `Err(String)` - An error message if resolution fails
pub fn resolve_path(path: &str) -> Result<PathBuf, String> {
    // Create instances of all resolvers
    let resolvers: Vec<Box<dyn PathResolver>> = vec![
        Box::new(GoPathResolver::new()),
        Box::new(JavaScriptPathResolver::new()),
        Box::new(RustPathResolver::new()),
    ];

    // Find the appropriate resolver based on the path prefix
    for resolver in resolvers {
        let prefix = resolver.prefix();
        if !prefix.ends_with(':') {
            // Internal sanity check
            eprintln!(
                "Warning: PathResolver prefix '{}' does not end with ':'",
                prefix
            );
            continue;
        }

        if let Some(full_path_after_prefix) = path.strip_prefix(prefix) {
            // 1. Split the path into module name and optional subpath
            let (module_name, subpath_opt) = resolver
                .split_module_and_subpath(full_path_after_prefix)
                .map_err(|e| {
                    format!(
                        "Failed to parse path '{}' for prefix '{}': {}",
                        full_path_after_prefix, prefix, e
                    )
                })?;

            // 2. Resolve the base directory of the module
            let module_base_path = resolver.resolve(&module_name).map_err(|e| {
                format!(
                    "Failed to resolve module '{}' for prefix '{}': {}",
                    module_name, prefix, e
                )
            })?;

            // 3. Combine base path with subpath if it exists
            let final_path = match subpath_opt {
                Some(sub) if !sub.is_empty() => {
                    // Ensure subpath is treated as relative
                    let relative_subpath = Path::new(&sub)
                        .strip_prefix("/")
                        .unwrap_or_else(|_| Path::new(&sub));
                    module_base_path.join(relative_subpath)
                }
                _ => module_base_path, // No subpath or empty subpath
            };

            return Ok(final_path);
        }
    }

    // If no special prefix, return the path as is
    Ok(PathBuf::from(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_path_regular() {
        let path = "/some/regular/path";
        let result = resolve_path(path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from(path));
    }
}
