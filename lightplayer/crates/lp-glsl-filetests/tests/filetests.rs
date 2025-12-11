//! Test discovery and execution for GLSL filetests.

use anyhow::Result;
use lp_glsl_filetests::run_filetest;
use std::env;
use std::path::PathBuf;
use walkdir::WalkDir;

#[test]
fn filetests() -> Result<()> {
    let filetests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("filetests");

    for entry in WalkDir::new(&filetests_dir) {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("glsl") {
            run_filetest(path)?;
        }
    }
    Ok(())
}
