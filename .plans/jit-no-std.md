# Plan: Making cranelift-jit Support no_std

## Overview

Make `cranelift-jit` work in `no_std` environments while maintaining backward compatibility. The key insight is that the existing `JITMemoryProvider` trait already provides the right abstraction - we just need to:

1. Gate `std`-dependent code behind feature flags
2. Make `SystemMemoryProvider` and `ArenaMemoryProvider` optional (require OS)
3. Allow users to provide custom memory providers for `no_std` environments
4. Replace `std` dependencies with `core`/`alloc` equivalents

## Motivation

- Light Player needs JIT compilation in embedded/bare-metal environments
- Many embedded systems have executable memory but no OS memory protection APIs
- Users should be able to provide pre-allocated executable memory regions

## Current State Analysis

### What `cranelift-module` Already Does

✅ `cranelift-module` already supports `no_std` via feature flags:
- `default = ["std"]`
- `std` feature enables full functionality
- `core` feature uses `hashbrown` instead of `std::collections`
- Uses `#![no_std]` with conditional `extern crate alloc as std`

### Current `std` Dependencies in `cranelift-jit`

From code analysis:

1. **Collections**:
   - `std::collections::HashMap` → Can use `hashbrown::HashMap` in no_std
   - `Vec`, `Box`, `String` → Available via `alloc`

2. **I/O & Error Handling**:
   - `std::io::Error`, `std::io::Write` → Need to handle carefully
   - `std::error::Error` trait → Can use custom error handling

3. **Platform-Specific (dlsym, memory protection)**:
   - `std::ffi::CString` (for dlsym)
   - `std::env`, `std::fs`, `std::process` (for perf integration)
   - `libc::dlsym` (Unix symbol lookup)
   - Windows `GetProcAddress` (Windows symbol lookup)
   - These should all be gated behind `std` or `system-memory` feature

4. **External Dependencies**:
   - `anyhow` - supports `no_std` via features
   - `region` - provides memory protection (OS-dependent)
   - `libc` - available in `no_std`
   - `wasmtime-jit-icache-coherence` - need to verify
   - `windows-sys` - Windows only, should be gated

### What Can Stay, What Must Go

**Can work in `no_std`:**
- ✅ `JITBuilder` (with custom memory provider)
- ✅ `JITModule` core functionality
- ✅ `JITMemoryProvider` trait (perfect abstraction!)
- ✅ `CompiledBlob` and relocations
- ✅ Symbol management (with custom lookup functions)

**Requires `std` or `system-memory` feature:**
- ❌ `SystemMemoryProvider` (uses OS allocator and mprotect)
- ❌ `ArenaMemoryProvider` (uses `region::alloc`)
- ❌ `lookup_with_dlsym` (requires libc/Windows APIs)
- ❌ Perf integration (`/tmp/perf-*.map` files)

## Architecture & Design Decisions

### 1. Feature Flag Structure

```toml
[features]
default = ["std"]

# Full std support with system memory providers
std = [
    "cranelift-codegen/std",
    "cranelift-module/std",
    "cranelift-control/std",
    "cranelift-entity/std",
    "anyhow/std",
    "system-memory",
]

# Core no_std support (requires custom memory provider)
core = [
    "cranelift-codegen/core",
    "cranelift-module/core",
    "hashbrown",
]

# System memory providers (requires OS)
system-memory = ["std"]

# SELinux fix (requires mmap, implies system-memory)
selinux-fix = ["memmap2", "system-memory"]

# Wasmtime unwinder support
wasmtime-unwinder = ["dep:wasmtime-unwinder"]
```

### 2. Memory Provider Strategy

The existing `JITMemoryProvider` trait is **already perfect**:

```rust
pub trait JITMemoryProvider {
    fn allocate_readexec(&mut self, size: usize, align: u64) -> io::Result<*mut u8>;
    fn allocate_readwrite(&mut self, size: usize, align: u64) -> io::Result<*mut u8>;
    fn allocate_readonly(&mut self, size: usize, align: u64) -> io::Result<*mut u8>;
    unsafe fn free_memory(&mut self);
    fn finalize(&mut self, branch_protection: BranchProtection) -> ModuleResult<()>;
}
```

**For `no_std` users**: Implement this trait with:
- Pre-allocated memory regions (from linker script or bootloader)
- Custom allocator that understands memory permissions
- Bare-metal HAL that can configure MMU/MPU

**For `std` users**: Continue using `SystemMemoryProvider` or `ArenaMemoryProvider`

### 3. Error Handling Strategy

Keep using `std::io::Error` as the error type for memory allocation, but:

1. In `no_std`, provide a minimal `io::Error` wrapper:
   ```rust
   #[cfg(not(feature = "std"))]
   pub mod io {
       pub use core2::io::Error;
       pub use core2::io::ErrorKind;
   }
   ```
   OR define a custom error type that works in both modes.

2. `ModuleError` already exists in `cranelift-module` and supports `no_std`

### 4. Symbol Resolution Strategy

Currently uses `dlsym` (Unix) or `GetProcAddress` (Windows) to lookup symbols.

**For `no_std`:**
- User provides symbol resolution via `JITBuilder::symbol()` and `symbol_lookup_fn()`
- No default symbol lookup (no `dlsym`)
- All external symbols must be explicitly registered

**For `std`:**
- Keep existing `dlsym`/`GetProcAddress` as default fallback
- Gate behind `#[cfg(feature = "system-memory")]`

## Implementation Plan

### Phase 1: Basic no_std Infrastructure

1. **Update `Cargo.toml`**:
   ```toml
   [dependencies]
   cranelift-module = { workspace = true }
   cranelift-native = { workspace = true }
   cranelift-codegen = { workspace = true }
   cranelift-entity = { workspace = true }
   cranelift-control = { workspace = true }
   hashbrown = { workspace = true, optional = true }
   anyhow = { workspace = true }
   target-lexicon = { workspace = true }
   log = { workspace = true }
   
   # System-dependent (only with std)
   region = { version = "3.0.2", optional = true }
   libc = { workspace = true, optional = true }
   wasmtime-jit-icache-coherence = { workspace = true }
   wasmtime-unwinder = { workspace = true, optional = true, features = ["cranelift"] }
   memmap2 = { version = "0.2.1", optional = true }
   
   [target.'cfg(windows)'.dependencies.windows-sys]
   workspace = true
   optional = true
   features = [...]
   
   [features]
   default = ["std"]
   std = ["cranelift-codegen/std", "cranelift-module/std", "system-memory", "anyhow/std"]
   core = ["cranelift-codegen/core", "cranelift-module/core", "hashbrown"]
   system-memory = ["std", "region", "libc", "windows-sys"]
   selinux-fix = ["memmap2", "system-memory"]
   wasmtime-unwinder = ["dep:wasmtime-unwinder"]
   ```

2. **Update `lib.rs`**:
   ```rust
   #![deny(missing_docs, unreachable_pub)]
   #![expect(unsafe_op_in_unsafe_fn, reason = "crate isn't migrated yet")]
   #![no_std]
   
   #[cfg(not(feature = "std"))]
   #[macro_use]
   extern crate alloc as std;
   #[cfg(feature = "std")]
   #[macro_use]
   extern crate std;
   
   mod backend;
   mod compiled_blob;
   mod memory;
   
   pub use crate::backend::{JITBuilder, JITModule};
   pub use crate::memory::{BranchProtection, JITMemoryProvider};
   
   #[cfg(feature = "system-memory")]
   pub use crate::memory::{ArenaMemoryProvider, SystemMemoryProvider};
   
   pub const VERSION: &str = env!("CARGO_PKG_VERSION");
   ```

### Phase 2: Update backend.rs

1. **Replace std imports**:
   ```rust
   #[cfg(not(feature = "std"))]
   use hashbrown::HashMap;
   #[cfg(feature = "std")]
   use std::collections::HashMap;
   
   use core::cell::RefCell;
   use core::ptr;
   use std::boxed::Box;
   use std::vec::Vec;
   use std::string::String;
   
   #[cfg(feature = "system-memory")]
   use std::ffi::CString;
   ```

2. **Gate dlsym lookups**:
   ```rust
   impl JITBuilder {
       pub fn with_isa(...) -> Self {
           let symbols = HashMap::new();
           
           #[cfg(feature = "system-memory")]
           let lookup_symbols = vec![Box::new(lookup_with_dlsym) as Box<_>];
           
           #[cfg(not(feature = "system-memory"))]
           let lookup_symbols = vec![];
           
           Self { ... }
       }
   }
   
   #[cfg(all(not(windows), feature = "system-memory"))]
   fn lookup_with_dlsym(name: &str) -> Option<*const u8> { ... }
   
   #[cfg(all(windows, feature = "system-memory"))]
   fn lookup_with_dlsym(name: &str) -> Option<*const u8> { ... }
   ```

3. **Gate perf integration**:
   ```rust
   fn record_function_for_perf(&self, ptr: *mut u8, size: usize, name: &str) {
       #[cfg(all(unix, feature = "std"))]
       {
           if ::std::env::var_os("PERF_BUILDID_DIR").is_some() {
               // ... perf recording logic
           }
       }
       #[cfg(not(all(unix, feature = "std")))]
       {
           let _ = (ptr, size, name); // suppress unused warnings
       }
   }
   ```

4. **Update JITModule::new()**:
   ```rust
   pub fn new(builder: JITBuilder) -> Self {
       assert!(!builder.isa.flags().is_pic(), "cranelift-jit needs is_pic=false");
       
       let memory = builder.memory.unwrap_or_else(|| {
           #[cfg(feature = "system-memory")]
           {
               Box::new(SystemMemoryProvider::new())
           }
           #[cfg(not(feature = "system-memory"))]
           {
               panic!("No memory provider specified. In no_std mode, you must provide a custom JITMemoryProvider via JITBuilder::memory_provider()")
           }
       });
       
       Self { ... }
   }
   ```

5. **Handle slice operations**:
   ```rust
   // This is fine - slice operations work in no_std
   let mem = unsafe { core::slice::from_raw_parts_mut(ptr, size) };
   mem.copy_from_slice(compiled_code.code_buffer());
   ```

6. **Handle alloc_error**:
   ```rust
   #[cfg(feature = "std")]
   use std::alloc::handle_alloc_error;
   
   #[cfg(not(feature = "std"))]
   fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
       panic!("allocation failed: {:?}", layout);
   }
   
   // Then in define_data:
   if ptr.is_null() {
       handle_alloc_error(
           core::alloc::Layout::from_size_align(...).unwrap()
       );
   }
   ```

### Phase 3: Update memory modules

1. **Update `memory/mod.rs`**:
   ```rust
   use cranelift_module::{ModuleError, ModuleResult};
   
   #[cfg(feature = "system-memory")]
   mod arena;
   #[cfg(feature = "system-memory")]
   mod system;
   
   #[cfg(feature = "system-memory")]
   pub use arena::ArenaMemoryProvider;
   #[cfg(feature = "system-memory")]
   pub use system::SystemMemoryProvider;
   
   /// Type of branch protection to apply to executable memory.
   #[derive(Clone, Copy, Debug, PartialEq)]
   pub enum BranchProtection {
       None,
       BTI,
   }
   
   /// A provider of memory for the JIT.
   pub trait JITMemoryProvider {
       fn allocate_readexec(&mut self, size: usize, align: u64) -> core2::io::Result<*mut u8>;
       fn allocate_readwrite(&mut self, size: usize, align: u64) -> core2::io::Result<*mut u8>;
       fn allocate_readonly(&mut self, size: usize, align: u64) -> core2::io::Result<*mut u8>;
       unsafe fn free_memory(&mut self);
       fn finalize(&mut self, branch_protection: BranchProtection) -> ModuleResult<()>;
   }
   
   #[cfg(feature = "system-memory")]
   pub(crate) fn set_readable_and_executable(...) -> ModuleResult<()> {
       // ... existing implementation
   }
   ```

2. **Keep `system.rs` and `arena.rs`** - They only compile when `feature = "system-memory"` is enabled

### Phase 4: Update compiled_blob.rs

1. **Replace std imports**:
   ```rust
   use core::ptr::write_unaligned;
   
   // in perform_relocations, the std::ptr::write_unaligned is already imported locally
   ```

### Phase 5: Handle Dependencies

1. **Check `wasmtime-jit-icache-coherence`**:
   - Verify it supports `no_std`
   - If not, make it optional or create a `no_std` compatible version
   - For embedded, might need platform-specific cache coherence

2. **Handle `log` crate**:
   - Already supports `no_std`

3. **Handle `anyhow`**:
   - Add `default-features = false` when `std` feature is off
   - Use `anyhow` features: `anyhow = { workspace = true, default-features = false, features = ["std"] }`

4. **Add `core2` or equivalent**:
   - For `io::Error` support in `no_std`
   - Alternative: define minimal error types in the crate itself

### Phase 6: Update `cranelift-native`

Check if `cranelift-native` needs to support `no_std`:
- It uses CPUID/detection which might be platform-specific
- For embedded, user might need to construct ISA manually
- Consider making `JITBuilder::with_isa()` the primary no_std entry point

## Testing Strategy

### Unit Tests

1. **Compile tests**:
   ```bash
   # Test no_std builds
   cargo check --no-default-features --features core
   
   # Test std builds (should still work)
   cargo check --features std
   cargo test --features std
   ```

2. **Mock memory provider test**:
   ```rust
   #[cfg(test)]
   mod tests {
       struct MockMemoryProvider {
           buffer: Vec<u8>,
           position: usize,
       }
       
       impl JITMemoryProvider for MockMemoryProvider {
           // ... implementation
       }
       
       #[test]
       fn test_jit_with_custom_provider() {
           let provider = MockMemoryProvider::new(1024 * 1024);
           let mut builder = JITBuilder::with_isa(..., default_libcall_names());
           builder.memory_provider(Box::new(provider));
           // ... test compilation
       }
   }
   ```

### Integration Tests

1. **Example no_std JIT**:
   Create `examples/no-std-jit/` (as a separate crate with `no_std`):
   ```rust
   #![no_std]
   #![no_main]
   
   extern crate alloc;
   
   struct StaticMemoryProvider {
       // Use static muts or linker-provided regions
   }
   
   #[no_mangle]
   pub extern "C" fn main() -> i32 {
       let provider = StaticMemoryProvider::new();
       let mut builder = JITBuilder::with_isa(...);
       builder.memory_provider(Box::new(provider));
       
       // Compile and run a simple function
       // ...
   }
   ```

2. **Embedded target test**:
   - Test on `thumbv7em-none-eabihf` or similar
   - Use QEMU for CI testing

## Documentation

### 1. Update README

Add section:

```markdown
## no_std Support

`cranelift-jit` supports `no_std` environments. To use it without `std`:

1. Disable default features: `cranelift-jit = { version = "...", default-features = false, features = ["core"] }`
2. Provide a custom memory provider implementing `JITMemoryProvider`
3. Register all external symbols via `JITBuilder::symbol()` (no dlsym in no_std)

See `examples/no-std-jit/` for a complete example.
```

### 2. Add rustdoc examples

```rust
/// # no_std Usage
///
/// In `no_std` environments, you must provide a custom memory provider:
///
/// ```ignore
/// struct MyMemoryProvider { /* ... */ }
/// impl JITMemoryProvider for MyMemoryProvider { /* ... */ }
///
/// let mut builder = JITBuilder::with_isa(isa, libcall_names);
/// builder.memory_provider(Box::new(MyMemoryProvider::new()));
/// builder.symbol("external_func", external_func as *const u8);
/// let module = JITModule::new(builder);
/// ```
```

### 3. Document memory provider requirements

In `JITMemoryProvider` trait docs:

```rust
/// # Memory Permissions
///
/// - `allocate_readexec`: Must return memory that will become executable after `finalize()`
/// - `allocate_readonly`: Must return memory that will become read-only after `finalize()`
/// - `allocate_readwrite`: Must return memory that remains writable
///
/// In `no_std` environments, you may need to:
/// - Pre-allocate memory regions with appropriate permissions
/// - Use platform HAL to configure MPU/MMU
/// - Ensure cache coherence for executable memory
```

## Example: no_std Memory Provider

```rust
/// Example memory provider for bare-metal systems
pub struct BareMetalMemoryProvider {
    exec_region: &'static mut [u8],
    readonly_region: &'static mut [u8],
    readwrite_region: &'static mut [u8],
    exec_pos: usize,
    readonly_pos: usize,
    readwrite_pos: usize,
}

impl BareMetalMemoryProvider {
    /// Create from linker-provided memory regions
    ///
    /// # Safety
    ///
    /// The memory regions must be valid, non-overlapping, and have
    /// appropriate permissions configured (or be configurable).
    pub unsafe fn from_linker_symbols() -> Self {
        extern "C" {
            static mut __jit_exec_start: u8;
            static mut __jit_exec_end: u8;
            static mut __jit_ro_start: u8;
            static mut __jit_ro_end: u8;
            static mut __jit_rw_start: u8;
            static mut __jit_rw_end: u8;
        }
        
        let exec_size = &__jit_exec_end as *const u8 as usize 
                      - &__jit_exec_start as *const u8 as usize;
        let ro_size = &__jit_ro_end as *const u8 as usize 
                    - &__jit_ro_start as *const u8 as usize;
        let rw_size = &__jit_rw_end as *const u8 as usize 
                    - &__jit_rw_start as *const u8 as usize;
        
        Self {
            exec_region: core::slice::from_raw_parts_mut(&mut __jit_exec_start, exec_size),
            readonly_region: core::slice::from_raw_parts_mut(&mut __jit_ro_start, ro_size),
            readwrite_region: core::slice::from_raw_parts_mut(&mut __jit_rw_start, rw_size),
            exec_pos: 0,
            readonly_pos: 0,
            readwrite_pos: 0,
        }
    }
}

impl JITMemoryProvider for BareMetalMemoryProvider {
    fn allocate_readexec(&mut self, size: usize, align: u64) -> io::Result<*mut u8> {
        let align = align as usize;
        self.exec_pos = (self.exec_pos + align - 1) & !(align - 1);
        
        if self.exec_pos + size > self.exec_region.len() {
            return Err(io::Error::new(
                io::ErrorKind::OutOfMemory,
                "exec memory exhausted"
            ));
        }
        
        let ptr = self.exec_region[self.exec_pos..].as_mut_ptr();
        self.exec_pos += size;
        Ok(ptr)
    }
    
    fn allocate_readwrite(&mut self, size: usize, align: u64) -> io::Result<*mut u8> {
        // Similar to allocate_readexec but for readwrite_region
        todo!()
    }
    
    fn allocate_readonly(&mut self, size: usize, align: u64) -> io::Result<*mut u8> {
        // Similar to allocate_readexec but for readonly_region
        todo!()
    }
    
    unsafe fn free_memory(&mut self) {
        // Reset positions
        self.exec_pos = 0;
        self.readonly_pos = 0;
        self.readwrite_pos = 0;
    }
    
    fn finalize(&mut self, branch_protection: BranchProtection) -> ModuleResult<()> {
        // On bare metal, might need to:
        // 1. Flush data cache for code regions
        // 2. Invalidate instruction cache
        // 3. Configure MPU if available
        
        #[cfg(target_arch = "arm")]
        unsafe {
            // Example: ARM Cortex-M cache operations
            core::arch::asm!(
                "dsb",    // Data Synchronization Barrier
                "isb",    // Instruction Synchronization Barrier
            );
        }
        
        Ok(())
    }
}
```

## Linker Script Example

```ld
/* linker.ld */
MEMORY
{
    FLASH : ORIGIN = 0x08000000, LENGTH = 1M
    RAM   : ORIGIN = 0x20000000, LENGTH = 128K
    JIT_EXEC : ORIGIN = 0x20020000, LENGTH = 64K
    JIT_RO   : ORIGIN = 0x20030000, LENGTH = 16K
    JIT_RW   : ORIGIN = 0x20034000, LENGTH = 16K
}

SECTIONS
{
    .jit_exec (NOLOAD) : {
        __jit_exec_start = .;
        . = . + 64K;
        __jit_exec_end = .;
    } > JIT_EXEC
    
    .jit_ro (NOLOAD) : {
        __jit_ro_start = .;
        . = . + 16K;
        __jit_ro_end = .;
    } > JIT_RO
    
    .jit_rw (NOLOAD) : {
        __jit_rw_start = .;
        . = . + 16K;
        __jit_rw_end = .;
    } > JIT_RW
}
```

## Migration Path for Existing Users

**No breaking changes needed!**

- Default feature set includes `std`
- `SystemMemoryProvider` remains the default
- All existing code continues to work

Optional: Users can opt into `no_std` by:
```toml
cranelift-jit = { version = "...", default-features = false, features = ["core"] }
```

## Open Questions

1. **`wasmtime-jit-icache-coherence`**: Does it support `no_std`? 
   - May need conditional compilation or platform-specific implementations
   - For ARM Cortex-M: manual `DSB`/`ISB` instructions
   - For RISC-V: `fence.i` instruction

2. **Error handling**: Use `core2::io` or define custom error types?
   - Recommend: Use custom error type to avoid external dependency
   - Or make `core2` an optional dependency

3. **Allocator requirements**: Should we require `alloc`?
   - Yes - we need `Vec`, `Box`, `HashMap` etc.
   - Document minimum allocator requirements

4. **cranelift-native**: How to handle in `no_std`?
   - Make it optional
   - Require users to construct `TargetIsa` manually in `no_std`
   - Document how to get target info in bare-metal

## Success Criteria

- [ ] `cargo check --no-default-features --features core` passes
- [ ] `cargo test --features std` passes (no regressions)
- [ ] Example bare-metal JIT compiles for ARM/RISC-V target
- [ ] Documentation clearly explains `no_std` usage
- [ ] No breaking changes for existing users

## Timeline Estimate

- Phase 1-2 (basic no_std): 2-3 days
- Phase 3-4 (memory/compiled_blob): 1-2 days  
- Phase 5 (dependencies): 1-2 days
- Phase 6 (testing & docs): 2-3 days
- **Total: ~1-2 weeks**

## Related Work

- Similar to how `cranelift-module` already supports `no_std`
- Follow patterns from other `no_std` JIT systems
- Consider compatibility with `embedded-hal` ecosystem

