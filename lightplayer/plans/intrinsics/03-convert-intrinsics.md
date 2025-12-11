# Stage 03: Convert Intrinsics to Fixed-Point

## Overview

Implement the conversion of intrinsic functions (like `__lp_sin`, `__lp_cos`) to fixed-point. Intrinsics are compiled lazily during codegen and added to the module. When fixed-point mode is enabled, we need to convert them before the main function is converted.

## Problem

1. Intrinsics are compiled lazily during codegen when first needed
2. Intrinsics are added to the module as internal functions
3. When fixed-point transform runs, it only converts the main function
4. Intrinsics remain in float format, causing type mismatches when called

## Goals

- Convert intrinsic functions to fixed-point when fixed-point mode is enabled
- Ensure intrinsics are converted before main function that calls them
- Handle recursive/converting dependencies (intrinsics calling other intrinsics)
- Cache converted intrinsics to avoid re-conversion
- Ensure only needed intrinsics are compiled

## Implementation

### 1. Collect Intrinsic Function Calls

**File**: `crates/lp-glsl/src/transform/fixed32/mod.rs`

Add helper to collect intrinsic function names from a function:

```rust
/// Collect all intrinsic function names that are called in a function.
///
/// Intrinsics are identified by the `__lp_` prefix.
pub fn collect_intrinsic_calls(func: &Function) -> Vec<String> {
    let mut intrinsics = Vec::new();

    for block in func.layout.blocks() {
        for inst in func.layout.block_insts(block) {
            if let InstructionData::Call { func_ref, .. } = &func.dfg.insts[inst] {
                // Check if this is an internal function call
                if !func.dfg.ext_funcs.contains_key(*func_ref) {
                    // Get function name from FuncRef
                    // This requires access to module to map FuncRef -> FuncId -> name
                    // For now, we'll need to pass module or track this differently
                }
            }
        }
    }

    intrinsics
}
```

**Alternative**: Collect intrinsic calls during codegen and store in a set.

### 2. Track Intrinsic Calls During Codegen

**File**: `crates/lp-glsl/src/codegen/context.rs`

Add tracking of intrinsic function calls:

```rust
pub struct CodegenContext<'a> {
    // ... existing fields ...

    // NEW: Track which intrinsics are called
    pub called_intrinsics: HashSet<String>,
}
```

**File**: `crates/lp-glsl/src/intrinsics/loader.rs`

When an intrinsic is created, track it:

```rust
pub fn get_or_create_intrinsic(
    libcall_name: &str,
    ctx: &mut CodegenContext,
) -> Result<FuncRef, GlslError> {
    // ... existing code ...

    // Track that this intrinsic was called
    ctx.called_intrinsics.insert(intrinsic_name.to_string());

    // ... rest of code ...
}
```

### 3. Convert Intrinsics Before Fixed-Point Transform

**File**: `crates/lp-glsl/src/jit.rs`

After codegen completes, convert intrinsics:

```rust
pub fn compile_detailed(...) -> Result<*const u8, GlslError> {
    // ... existing codegen ...

    // NEW: Convert intrinsic functions to fixed-point if enabled
    if let Some(format) = self.fixed_point_format {
        // Get list of called intrinsics from codegen context
        let called_intrinsics = /* get from codegen context */;

        // Convert each intrinsic function
        for intrinsic_name in called_intrinsics {
            convert_intrinsic_in_module(
                &mut self.module,
                &intrinsic_name,
                format,
            )?;
        }
    }

    // Now convert main function
    if let Some(format) = self.fixed_point_format {
        crate::transform::fixed32::convert_floats_to_fixed(&mut self.ctx.func, format)?;
    }

    // ... rest of compilation ...
}
```

### 4. Helper to Convert Intrinsic in Module

**File**: `crates/lp-glsl/src/transform/fixed32/mod.rs`

```rust
/// Convert an intrinsic function in the module from float to fixed-point.
///
/// This function:
/// 1. Gets the function from the module by name
/// 2. Extracts the function body
/// 3. Converts it to fixed-point
/// 4. Replaces it in the module
pub fn convert_intrinsic_in_module(
    module: &mut dyn Module,
    func_name: &str,
    format: FixedPointFormat,
) -> Result<(), GlslError> {
    // Get function ID from name
    let func_id = if let Some(FuncOrDataId::Func(id)) = module.get_name(func_name) {
        id
    } else {
        // Function not found - this shouldn't happen if called correctly
        return Err(GlslError::new(
            ErrorCode::E0400,
            format!("Intrinsic function {} not found in module", func_name),
        ));
    };

    // Get function signature from module declarations
    let declarations = module.declarations();
    let func_decl = declarations.functions.get(func_id)
        .ok_or_else(|| GlslError::new(
            ErrorCode::E0400,
            format!("Function declaration not found for {}", func_name),
        ))?;

    // Challenge: We need to get the function body (Function object)
    // Module API doesn't expose this directly

    // Option 1: Store functions separately during compilation
    // Option 2: Extract from module's internal storage (if possible)
    // Option 3: Recompile from GLSL source (simpler but less efficient)

    // For now, we'll use Option 3: Recompile from GLSL source
    // This is acceptable because intrinsics are small and cached

    // Get GLSL source for this intrinsic
    let glsl_source = get_intrinsic_glsl_source(func_name)?;

    // Compile intrinsic function
    let isa = module.isa();
    let compiled_functions = crate::intrinsics::compiler::compile_intrinsic_functions(
        glsl_source,
        isa,
    )?;

    // Get the function we need
    let func = compiled_functions.get(func_name)
        .ok_or_else(|| GlslError::new(
            ErrorCode::E0400,
            format!("Function {} not found in compiled intrinsics", func_name),
        ))?;

    // Convert to fixed-point
    let converted_func = crate::transform::fixed32::rewrite::rewrite_function(func, format)?;

    // Replace function in module
    let mut ctx = Context::for_function(converted_func);
    module.define_function(func_id, &mut ctx)
        .map_err(|e| GlslError::new(
            ErrorCode::E0400,
            format!("Failed to replace intrinsic {}: {}", func_name, e),
        ))?;

    Ok(())
}

/// Get GLSL source for an intrinsic function.
fn get_intrinsic_glsl_source(func_name: &str) -> Result<&'static str, GlslError> {
    // Determine which file contains this intrinsic
    let file_name = match func_name {
        name if name.starts_with("__lp_sin") || name.starts_with("__lp_cos")
            || name.starts_with("__lp_tan") => "trig",
        // Add more mappings as needed
        _ => return Err(GlslError::new(
            ErrorCode::E0400,
            format!("Unknown intrinsic function: {}", func_name),
        )),
    };

    match file_name {
        "trig" => Ok(include_str!("../../intrinsics/trig.glsl")),
        _ => Err(GlslError::new(
            ErrorCode::E0400,
            format!("Unknown intrinsic file: {}", file_name),
        )),
    }
}
```

### 5. Handle Intrinsic Dependencies

Intrinsics may call other intrinsics (e.g., `__lp_cos` calls `__lp_sin`). We need to convert them in dependency order:

```rust
/// Convert all intrinsic functions, handling dependencies.
fn convert_all_intrinsics(
    module: &mut dyn Module,
    called_intrinsics: &HashSet<String>,
    format: FixedPointFormat,
) -> Result<(), GlslError> {
    // Build dependency graph
    // For now, convert all functions in the intrinsic file together
    // This ensures dependencies are handled

    // Group intrinsics by file
    let mut intrinsics_by_file: HashMap<&str, Vec<String>> = HashMap::new();
    for intrinsic in called_intrinsics {
        let file = get_intrinsic_file(intrinsic)?;
        intrinsics_by_file.entry(file).or_insert_with(Vec::new).push(intrinsic.clone());
    }

    // Convert each file's intrinsics together
    for (file, intrinsics) in intrinsics_by_file {
        let glsl_source = get_intrinsic_file_source(file)?;

        // Compile all functions in file
        let compiled = crate::intrinsics::compiler::compile_intrinsic_functions(
            glsl_source,
            module.isa(),
        )?;

        // Convert each function
        for intrinsic_name in intrinsics {
            if let Some(func) = compiled.get(&intrinsic_name) {
                let converted = crate::transform::fixed32::rewrite::rewrite_function(func, format)?;

                // Replace in module
                let func_id = /* get from module */;
                let mut ctx = Context::for_function(converted);
                module.define_function(func_id, &mut ctx)?;
            }
        }
    }

    Ok(())
}
```

### 6. Update Intrinsic Cache

**File**: `crates/lp-glsl/src/intrinsics/loader.rs`

Enhance cache to track fixed-point conversions:

```rust
pub struct IntrinsicCache {
    // Compiled functions (float version)
    pub compiled_functions: HashMap<String, Function>,
    // Function references in module
    pub module_func_refs: HashMap<String, FuncRef>,
    // NEW: Fixed-point converted functions
    pub fixed_point_functions: HashMap<String, Function>,
    // NEW: Track which format was used
    pub fixed_point_format: Option<FixedPointFormat>,
}
```

## Testing

### Test File: `lightplayer/crates/lp-glsl-filetests/filetests/fixed32/intrinsics_sin.glsl`

```glsl
// test compile
// test fixed32
// test run

float main() {
    float pi_2 = 1.570796327;  // π/2
    return sin(pi_2);  // Should be ~1.0
}

// Expected CLIF: __lp_sin function with i32 types (fixed-point)
// run: ~= 1.0  (tolerance: 0.01 for fixed-point)
```

### Test File: `lightplayer/crates/lp-glsl-filetests/filetests/fixed32/intrinsics_cos.glsl`

```glsl
// test compile
// test fixed32
// test run

float main() {
    return cos(0.0);  // Should be 1.0
}

// run: ~= 1.0
```

## Success Criteria

- [ ] Intrinsic functions are converted to fixed-point when fixed-point mode is enabled
- [ ] Intrinsics are converted before main function
- [ ] Intrinsic dependencies are handled correctly (cos calling sin)
- [ ] Only called intrinsics are compiled and converted
- [ ] Tests pass for sin, cos, etc. in fixed-point mode
- [ ] No F32 types remain in converted intrinsics

## Files to Modify

1. `crates/lp-glsl/src/transform/fixed32/mod.rs` - Add conversion helpers
2. `crates/lp-glsl/src/jit.rs` - Convert intrinsics before transform
3. `crates/lp-glsl/src/compiler.rs` - Convert intrinsics before transform
4. `crates/lp-glsl/src/intrinsics/loader.rs` - Enhance cache
5. `crates/lp-glsl/src/codegen/context.rs` - Track called intrinsics
6. `lightplayer/crates/lp-glsl-filetests/filetests/fixed32/intrinsics_*.glsl` - NEW test files

## Dependencies

- **Depends on**: Stage 01 (user functions), Stage 02 (internal call detection)
- **Enables**: Stage 04 (testing and validation)

## Notes

- Intrinsics are small, so recompiling from GLSL source is acceptable
- We may optimize later to cache converted functions
- Intrinsic dependencies (cos calling sin) are handled by converting entire file together
