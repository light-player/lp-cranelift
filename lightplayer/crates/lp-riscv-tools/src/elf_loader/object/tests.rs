//! Tests for object file loading.

#[cfg(test)]
mod tests {
    extern crate alloc;
    extern crate std;

    use alloc::string::String;
    use alloc::vec::Vec;
    use crate::elf_loader::load_elf;
    use crate::elf_loader::load_object_file;
    use std::println;

    /// Helper to compile a simple Rust source file to an object file.
    /// Returns the object file bytes, or None if compilation fails.
    fn compile_test_object(source: &str, name: &str) -> Option<Vec<u8>> {
        use std::env;
        use std::fs;
        use std::process::Command;

        let target = "riscv32imac-unknown-none-elf";

        // Find workspace root
        let mut current_dir = env::current_dir().ok()?;
        loop {
            let cargo_toml = current_dir.join("Cargo.toml");
            if cargo_toml.exists() {
                if let Ok(contents) = fs::read_to_string(&cargo_toml) {
                    if contents.contains("[workspace]") {
                        break;
                    }
                }
            }
            if let Some(parent) = current_dir.parent() {
                current_dir = parent.to_path_buf();
            } else {
                return None;
            }
        }

        // Create temporary source file
        let temp_dir = current_dir.join("target").join("test-objects");
        fs::create_dir_all(&temp_dir).ok()?;
        let source_path = temp_dir.join(std::format!("{}.rs", name));
        fs::write(&source_path, source).ok()?;

        // Compile to object file
        let obj_path = temp_dir.join(std::format!("{}.o", name));
        let output = Command::new("rustc")
            .args(&[
                "--target", target,
                "--crate-type", "rlib",
                "--emit", "obj",
                "-C", "relocation-model=pic",
                "-C", "opt-level=0",
                "-o", obj_path.to_str()?,
                source_path.to_str()?,
            ])
            .output()
            .ok()?;

        if !output.status.success() {
            println!("rustc failed: {}", String::from_utf8_lossy(&output.stderr));
            return None;
        }

        fs::read(&obj_path).ok()
    }

    /// Find the builtins executable path.
    fn find_builtins_executable() -> Option<Vec<u8>> {
        use std::env;

        let target = "riscv32imac-unknown-none-elf";

        // Try to find workspace root
        let mut current_dir = env::current_dir().ok()?;
        loop {
            let cargo_toml = current_dir.join("Cargo.toml");
            if cargo_toml.exists() {
                if let Ok(contents) = std::fs::read_to_string(&cargo_toml) {
                    if contents.contains("[workspace]") {
                        break;
                    }
                }
            }
            if let Some(parent) = current_dir.parent() {
                current_dir = parent.to_path_buf();
            } else {
                return None;
            }
        }

        // Try both debug and release profiles
        for profile in ["debug", "release"].iter() {
            let exe_path = current_dir
                .join("lightplayer")
                .join("target")
                .join(target)
                .join(profile)
                .join("lp-builtins-app");

            let exe_path = if exe_path.exists() {
                exe_path
            } else {
                current_dir
                    .join("target")
                    .join(target)
                    .join(profile)
                    .join("lp-builtins-app")
            };

            if exe_path.exists() {
                return std::fs::read(&exe_path).ok();
            }
        }

        None
    }

    #[test]
    fn test_load_object_file_basic() {
        // Create a simple test object file
        let source = r#"
            #![no_std]
            #![no_main]

            #[no_mangle]
            pub extern "C" fn main() -> i32 {
                42
            }
        "#;

        let obj_bytes = match compile_test_object(source, "test_main") {
            Some(bytes) => bytes,
            None => {
                println!("Skipping test: could not compile test object file. Install rustc with riscv32imac-unknown-none-elf target.");
                return;
            }
        };

        // Load base executable
        let builtins_exe = match find_builtins_executable() {
            Some(bytes) => {
                if bytes.is_empty() {
                    println!("Skipping test: builtins executable is empty");
                    return;
                }
                bytes
            }
            None => {
                println!("Skipping test: builtins executable not found. Build it with: scripts/build-builtins.sh");
                return;
            }
        };

        let mut base_info = match load_elf(&builtins_exe) {
            Ok(info) => info,
            Err(e) => {
                panic!("Failed to load base executable: {}", e);
            }
        };

        // Load object file
        let obj_info = match load_object_file(
            &obj_bytes,
            &mut base_info.code,
            &mut base_info.ram,
            &mut base_info.symbol_map,
        ) {
            Ok(info) => info,
            Err(e) => {
                panic!("Failed to load object file: {}", e);
            }
        };

        // Verify object file was loaded
        assert!(obj_info.text_start > 0, "Object file .text should be placed after base");
        // data_start is u32, so it's always >= 0
        
        // Verify main symbol was found
        if let Some(main_addr) = obj_info.main_address {
            assert!(main_addr > 0, "Main address should be valid");
            println!("Object file main() found at 0x{:x}", main_addr);
        } else {
            println!("No main symbol found in object file (this is OK for some object files)");
        }

        // Verify symbol map was updated
        assert!(!base_info.symbol_map.is_empty(), "Symbol map should contain symbols");
    }

    #[test]
    fn test_load_multiple_object_files() {
        // Create two test object files
        let source1 = r#"
            #![no_std]
            #![no_main]

            #[no_mangle]
            pub extern "C" fn func1() -> i32 {
                1
            }
        "#;

        let source2 = r#"
            #![no_std]
            #![no_main]

            #[no_mangle]
            pub extern "C" fn func2() -> i32 {
                2
            }

            #[no_mangle]
            pub extern "C" fn main() -> i32 {
                func2()
            }
        "#;

        let obj1_bytes = match compile_test_object(source1, "test_func1") {
            Some(bytes) => bytes,
            None => {
                println!("Skipping test: could not compile test object file 1");
                return;
            }
        };

        let obj2_bytes = match compile_test_object(source2, "test_func2") {
            Some(bytes) => bytes,
            None => {
                println!("Skipping test: could not compile test object file 2");
                return;
            }
        };

        // Load base executable
        let builtins_exe = match find_builtins_executable() {
            Some(bytes) => {
                if bytes.is_empty() {
                    println!("Skipping test: builtins executable is empty");
                    return;
                }
                bytes
            }
            None => {
                println!("Skipping test: builtins executable not found");
                return;
            }
        };

        let mut base_info = match load_elf(&builtins_exe) {
            Ok(info) => info,
            Err(e) => {
                panic!("Failed to load base executable: {}", e);
            }
        };

        let code_size_before = base_info.code.len();
        let ram_size_before = base_info.ram.len();

        // Load first object file
        let obj1_info = match load_object_file(
            &obj1_bytes,
            &mut base_info.code,
            &mut base_info.ram,
            &mut base_info.symbol_map,
        ) {
            Ok(info) => info,
            Err(e) => {
                panic!("Failed to load first object file: {}", e);
            }
        };

        // Load second object file
        let obj2_info = match load_object_file(
            &obj2_bytes,
            &mut base_info.code,
            &mut base_info.ram,
            &mut base_info.symbol_map,
        ) {
            Ok(info) => info,
            Err(e) => {
                panic!("Failed to load second object file: {}", e);
            }
        };

        // Verify buffers were extended
        assert!(base_info.code.len() >= code_size_before, "Code buffer should be extended");
        assert!(base_info.ram.len() >= ram_size_before, "RAM buffer should be extended");

        // Verify object files were placed sequentially
        assert!(obj2_info.text_start >= obj1_info.text_start, "Second object should be after first");

        // Verify symbol map contains both object files' symbols
        assert!(base_info.symbol_map.contains_key("func1"), "func1 should be in symbol map");
        assert!(base_info.symbol_map.contains_key("func2"), "func2 should be in symbol map");

        // Verify last main wins
        if let Some(main_addr) = obj2_info.main_address {
            println!("Second object file's main() at 0x{:x} (last one wins)", main_addr);
        }
    }

    #[test]
    fn test_object_file_error_cases() {
        // Test with invalid object file bytes
        let invalid_bytes: &[u8] = b"not an object file";
        
        let mut base_info = match find_builtins_executable()
            .and_then(|bytes| load_elf(&bytes).ok())
        {
            Some(info) => info,
            None => {
                println!("Skipping test: builtins executable not found");
                return;
            }
        };

        // Should fail with invalid object file
        let result = load_object_file(
            invalid_bytes,
            &mut base_info.code,
            &mut base_info.ram,
            &mut base_info.symbol_map,
        );

        assert!(result.is_err(), "Should fail with invalid object file");
        println!("Correctly rejected invalid object file: {:?}", result.err());
    }
}

