//! Build script for lp-glsl
//!
//! This script sets up the path to lp-builtins static library.
//! The library must be built manually with:
//!   cargo build --target riscv32imac-unknown-none-elf --package lp-builtins

#[cfg(feature = "emulator")]
fn main() {
    use std::env;

    let target = "riscv32imac-unknown-none-elf";
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    // Find workspace root
    let out_dir = env::var("OUT_DIR").unwrap();
    let workspace_root = find_workspace_root(&out_dir)
        .expect("Could not find workspace root (looking for Cargo.toml with [workspace])");

    // Path to lp-builtins crate (workspace root is already lightplayer/)
    let builtins_crate_path = workspace_root.join("crates").join("lp-builtins");

    // Path to the generated .a file
    let lib_path = workspace_root
        .join("target")
        .join(target)
        .join(&profile)
        .join("liblp_builtins.a");

    // Check if library exists and copy to OUT_DIR for compile-time inclusion
    if !lib_path.exists() {
        println!(
            "cargo:warning=lp-builtins library not found at: {}",
            lib_path.display()
        );
        println!(
            "cargo:warning=Build it manually with: cargo build --target {} --package lp-builtins",
            target
        );
        // Generate empty bytes if library doesn't exist
        let out_file = std::path::Path::new(&out_dir).join("lp_builtins_lib.rs");
        std::fs::write(&out_file, "pub const LP_BUILTINS_LIB_BYTES: &[u8] = &[];\n")
            .expect("Failed to write empty builtins lib file");
    } else {
        println!(
            "cargo:warning=lp-builtins library found at: {}",
            lib_path.display()
        );
        // Copy library to OUT_DIR
        let out_file = std::path::Path::new(&out_dir).join("liblp_builtins.a");
        std::fs::copy(&lib_path, &out_file).expect("Failed to copy lp-builtins library to OUT_DIR");

        // Generate a module that includes the library bytes
        let include_file = std::path::Path::new(&out_dir).join("lp_builtins_lib.rs");
        let include_path = out_file
            .strip_prefix(&out_dir)
            .expect("Failed to get relative path")
            .to_string_lossy()
            .replace('\\', "/");
        std::fs::write(
            &include_file,
            format!(
                "pub const LP_BUILTINS_LIB_BYTES: &[u8] = include_bytes!(\"{}\");\n",
                include_path
            ),
        )
        .expect("Failed to write builtins lib include file");
    }

    // Tell Cargo to rerun if lp-builtins source changes
    println!(
        "cargo:rerun-if-changed={}",
        builtins_crate_path.join("Cargo.toml").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        builtins_crate_path.join("src").display()
    );
    // Also rerun if the library changes (in case it's rebuilt externally)
    println!("cargo:rerun-if-changed={}", lib_path.display());
}

#[cfg(not(feature = "emulator"))]
fn main() {
    // No-op when emulator feature is disabled
}

/// Find the workspace root by looking for Cargo.toml with [workspace]
#[allow(dead_code)]
fn find_workspace_root(start: &str) -> Option<std::path::PathBuf> {
    use std::path::Path;
    let mut current = Path::new(start);

    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(contents) = std::fs::read_to_string(&cargo_toml) {
                if contents.contains("[workspace]") {
                    return Some(current.to_path_buf());
                }
            }
        }

        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            break;
        }
    }

    None
}
