//! Project loader for filesystem-based projects

use crate::error::Error;
use crate::fs::Filesystem;
use crate::project::config::ProjectConfig;
use alloc::{format, string::String, string::ToString, vec::Vec};

/// Load a project from the filesystem
///
/// Reads `/project.json` to get the top-level project configuration.
/// Node discovery is handled separately via `discover_nodes()`.
pub fn load_from_filesystem(fs: &dyn Filesystem) -> Result<ProjectConfig, Error> {
    // Read project.json
    let project_json = fs.read_file("/project.json")?;
    let project_str = core::str::from_utf8(&project_json)
        .map_err(|e| Error::Serialization(format!("Invalid UTF-8 in project.json: {}", e)))?;

    // Deserialize project config
    let config: ProjectConfig = serde_json::from_str(project_str)
        .map_err(|e| Error::Serialization(format!("Failed to parse project.json: {}", e)))?;

    Ok(config)
}

/// Discover all node directories in the project
///
/// Recursively scans `/src/` for directories ending with node suffixes
/// (`.shader`, `.texture`, `.output`, `.fixture`).
///
/// Returns a list of node IDs (full paths from project root with leading slash,
/// e.g., `["/src/my-shader.shader", "/src/nested/effects/rainbow.shader"]`).
pub fn discover_nodes(fs: &dyn Filesystem) -> Result<Vec<String>, Error> {
    let mut nodes = Vec::new();
    discover_nodes_recursive(fs, "/src", &mut nodes)?;
    Ok(nodes)
}

/// Recursively discover node directories
fn discover_nodes_recursive(
    fs: &dyn Filesystem,
    dir_path: &str,
    nodes: &mut Vec<String>,
) -> Result<(), Error> {
    // Try to list directory contents
    // If the directory doesn't exist, list_dir will return an error, which is fine
    let entries = match fs.list_dir(dir_path) {
        Ok(entries) => entries,
        Err(_) => {
            // Directory doesn't exist (e.g., src/ doesn't exist yet), that's okay
            return Ok(());
        }
    };

    for entry in entries {
        // Check if this entry is a node directory
        if is_node_directory(&entry) {
            // Extract node ID (full path from project root)
            if let Some(node_id) = extract_node_id(&entry) {
                nodes.push(node_id);
            }
        } else {
            // Check if it's a subdirectory (not a file)
            // We can't easily distinguish files from directories with the current Filesystem trait,
            // so we'll try to list it. If it fails, it's probably a file and we'll skip it.
            if let Ok(_) = fs.list_dir(&entry) {
                // It's a directory, recurse into it
                discover_nodes_recursive(fs, &entry, nodes)?;
            }
        }
    }

    Ok(())
}

/// Check if a path represents a node directory
///
/// Node directories end with one of: `.shader`, `.texture`, `.output`, `.fixture`
fn is_node_directory(path: &str) -> bool {
    path.ends_with(".shader")
        || path.ends_with(".texture")
        || path.ends_with(".output")
        || path.ends_with(".fixture")
}

/// Extract node ID from a directory path
///
/// Returns the full path from project root with leading slash.
/// For example, `/src/my-shader.shader` -> `/src/my-shader.shader`
fn extract_node_id(path: &str) -> Option<String> {
    // Ensure path starts with /
    if !path.starts_with('/') {
        return None;
    }

    // Path is already the node ID (full path from project root)
    Some(path.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::memory::InMemoryFilesystem;
    use alloc::string::ToString;

    #[test]
    fn test_load_from_filesystem() {
        let mut fs = InMemoryFilesystem::new();
        fs.write_file(
            "/project.json",
            br#"{"uid":"test-uid","name":"Test Project"}"#,
        )
        .unwrap();

        let config = load_from_filesystem(&fs).unwrap();
        assert_eq!(config.uid, "test-uid");
        assert_eq!(config.name, "Test Project");
    }

    #[test]
    fn test_load_from_filesystem_missing_file() {
        let fs = InMemoryFilesystem::new();
        let result = load_from_filesystem(&fs);
        assert!(result.is_err());
    }

    #[test]
    fn test_discover_nodes() {
        let mut fs = InMemoryFilesystem::new();
        // Create node directories by creating files in them
        fs.write_file("/src/shader1.shader/node.json", b"{}").unwrap();
        fs.write_file("/src/texture1.texture/node.json", b"{}").unwrap();
        fs.write_file("/src/output1.output/node.json", b"{}").unwrap();
        fs.write_file("/src/fixture1.fixture/node.json", b"{}").unwrap();

        let nodes = discover_nodes(&fs).unwrap();
        assert_eq!(nodes.len(), 4);
        assert!(nodes.contains(&"/src/shader1.shader".to_string()));
        assert!(nodes.contains(&"/src/texture1.texture".to_string()));
        assert!(nodes.contains(&"/src/output1.output".to_string()));
        assert!(nodes.contains(&"/src/fixture1.fixture".to_string()));
    }

    #[test]
    fn test_discover_nodes_nested() {
        let mut fs = InMemoryFilesystem::new();
        // Create nested node directories
        fs.write_file("/src/nested/effects/rainbow.shader/node.json", b"{}")
            .unwrap();
        fs.write_file("/src/nested/textures/bg.texture/node.json", b"{}")
            .unwrap();

        let nodes = discover_nodes(&fs).unwrap();
        assert_eq!(nodes.len(), 2);
        assert!(nodes.contains(&"/src/nested/effects/rainbow.shader".to_string()));
        assert!(nodes.contains(&"/src/nested/textures/bg.texture".to_string()));
    }

    #[test]
    fn test_discover_nodes_empty_src() {
        let fs = InMemoryFilesystem::new();
        let nodes = discover_nodes(&fs).unwrap();
        assert_eq!(nodes.len(), 0);
    }

    #[test]
    fn test_is_node_directory() {
        assert!(is_node_directory("/src/my-shader.shader"));
        assert!(is_node_directory("/src/my-texture.texture"));
        assert!(is_node_directory("/src/my-output.output"));
        assert!(is_node_directory("/src/my-fixture.fixture"));
        assert!(!is_node_directory("/src/regular-dir"));
        assert!(!is_node_directory("/src/file.txt"));
    }

    #[test]
    fn test_extract_node_id() {
        assert_eq!(
            extract_node_id("/src/my-shader.shader"),
            Some("/src/my-shader.shader".to_string())
        );
        assert_eq!(
            extract_node_id("/src/nested/effects/rainbow.shader"),
            Some("/src/nested/effects/rainbow.shader".to_string())
        );
        assert_eq!(extract_node_id("src/my-shader.shader"), None); // No leading slash
    }
}

