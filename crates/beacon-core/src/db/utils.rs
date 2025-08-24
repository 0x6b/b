//! Database utility functions for path handling.

use std::{env::current_dir, path::Path};

use crate::error::{PlannerError, Result};

impl super::Database {
    /// Canonicalize a directory path for search purposes using the same logic
    /// as plan creation
    pub fn canonicalize_directory_for_search(&self, directory: &str) -> Result<String> {
        let path = Path::new(directory);
        if path.is_absolute() {
            Ok(directory.to_string())
        } else {
            // Convert relative path to absolute
            let cwd = current_dir().map_err(|_| PlannerError::InvalidInput {
                field: "directory".to_string(),
                reason: "Cannot resolve current working directory to make path absolute"
                    .to_string(),
            })?;
            let absolute_path = cwd.join(path);
            // Normalize the path to resolve ".." and "." components without requiring the
            // path to exist
            let normalized_path = Self::normalize_path(&absolute_path);
            normalized_path
                .to_str()
                .map(String::from)
                .ok_or_else(|| PlannerError::InvalidInput {
                    field: "directory".to_string(),
                    reason: "Cannot convert path to string".to_string(),
                })
        }
    }

    /// Normalizes a path by resolving "." and ".." components without requiring
    /// the path to exist
    fn normalize_path(path: &Path) -> std::path::PathBuf {
        path.components()
            .fold(std::path::PathBuf::new(), |mut acc, component| {
                match component {
                    std::path::Component::CurDir => acc, // Skip "." components
                    std::path::Component::ParentDir => {
                        // Handle ".." by popping the last component if possible
                        acc.pop();
                        acc
                    }
                    _ => {
                        // Keep all other components (Normal, RootDir, Prefix)
                        acc.push(component);
                        acc
                    }
                }
            })
    }

    /// Ensures a directory path is absolute. Converts relative paths to
    /// absolute using the current working directory.
    pub(crate) fn ensure_absolute_directory(directory: Option<&str>) -> Result<Option<String>> {
        match directory {
            Some(dir) => {
                let path = Path::new(dir);
                if path.is_absolute() {
                    Ok(Some(dir.to_string()))
                } else {
                    // Convert relative path to absolute
                    let cwd = current_dir().map_err(|_| PlannerError::InvalidInput {
                        field: "directory".to_string(),
                        reason: "Cannot resolve current working directory to make path absolute"
                            .to_string(),
                    })?;
                    let absolute_path = cwd.join(path);
                    // Normalize the path to resolve ".." and "." components without requiring the
                    // path to exist
                    let normalized_path = Self::normalize_path(&absolute_path);
                    Ok(normalized_path.to_str().map(String::from))
                }
            }
            None => {
                // Use current working directory as default
                let cwd = current_dir().map_err(|_| PlannerError::InvalidInput {
                    field: "directory".to_string(),
                    reason: "Cannot determine current working directory".to_string(),
                })?;
                let normalized_cwd = Self::normalize_path(&cwd);
                Ok(normalized_cwd.to_str().map(String::from))
            }
        }
    }
}
