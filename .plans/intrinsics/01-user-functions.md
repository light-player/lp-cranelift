# Stage 01: Support Float -> Fixed User Functions

## Overview

This stage is a prerequisite for intrinsic function support. We need to ensure that user-defined GLSL functions can be converted from float to fixed-point correctly. Currently, the fixed-point transform only handles the main function, but user functions also need to be converted.

## Problem

When a GLSL shader has user-defined functions:

1. User functions are compiled separately and added to the module
2. Main function calls user functions via `FuncRef`
3. Fixed-point transform only converts the main function
4. User functions remain in float format, causing type mismatches

## Goals

- Convert all user-defined functions to fixed-point when fixed-point mode is enabled
- Ensure function calls between main and user functions work correctly
- Handle function signatures (parameters and return types) correctly
- Add comprehensive tests for user functions in fixed-point mode

## Implementation

### 1. Enhance Fixed-Point Transform to Handle Internal Function Calls

**File**: `crates/lp-glsl/src/transform/fixed32/converters/calls.rs`

Currently, `convert_call()` assumes all calls are external functions. We need to detect internal vs external calls:

```rust
pub(crate) fn convert_call(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    ext_func_map: &mut HashMap<FuncRef, FuncRef>,
    sig_map: &mut HashMap<SigRef, SigRef>,
    format: FixedPointFormat,
    block_map: &HashMap<Block, Block>,
    // NEW: Track internal function conversions
    internal_func_map: &mut HashMap<FuncId, FuncId>, // Maps old FuncId -> new FuncId
) -> Result<(), GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];

    if let InstructionData::Call { func_ref, args } = inst_data {
        // Check if this is an external or internal function call
        if old_func.dfg.ext_funcs.contains_key(*func_ref) {
            // External function - existing logic
            let new_func_ref = map_external_function(...)?;
            // ... rest of existing code
        } else {
            // Internal function call (user function or intrinsic)
            // For now, we'll handle this by converting signatures
            // The actual function conversion happens at module level

            // Get the function ID from the FuncRef
            // This requires tracking FuncRef -> FuncId mapping

            // Convert signature for the call
            let old_sig = /* get signature from func_ref */;
            let new_sig = convert_signature(old_sig, format);
            let new_sig_ref = builder.func.import_signature(new_sig);

            // Map arguments
            let old_args = args.as_slice(&old_func.dfg.value_lists);
            let new_args: Vec<Value> = old_args.iter().map(|&v| map_value(value_map, v)).collect();

            // For internal calls, we need to get the new FuncRef
            // This will be set up when we convert the called function
            // For now, we'll need to track this mapping

            // Emit call with converted signature
            // Note: This is incomplete - we need FuncId -> FuncId mapping
        }
    }
}
```

**Challenge**: Cranelift's `FuncRef` doesn't directly give us the `FuncId`. We need to track this mapping during codegen.

### 2. Track Function References During Codegen

**File**: `crates/lp-glsl/src/codegen/expr/function.rs`

When `declare_func_in_func` is called, we need to track the mapping from `FuncRef` to `FuncId`:

```rust
// Add to CodegenContext:
pub func_ref_to_id_map: HashMap<FuncRef, FuncId>,
```

**Alternative approach**: Instead of tracking FuncRef -> FuncId, we can convert all user functions at the module level before converting main function.

### 3. Convert User Functions at Module Level

**File**: `crates/lp-glsl/src/jit.rs`

After compiling all user functions but before converting main function:

```rust
fn translate(...) -> Result<(), GlslError> {
    // ... existing code to compile user functions ...

    // NEW: Convert user functions to fixed-point if enabled
    if let Some(format) = self.fixed_point_format {
        for user_func in &typed_ast.user_functions {
            let func_id = func_ids[&user_func.name];

            // Get the function from module
            // Convert it to fixed-point
            // Replace it in module

            // This requires access to the function's IR, which is stored in the module
            // We may need to extract it, convert it, and put it back
        }
    }

    // Then compile main function (which will also be converted)
}
```

**File**: `crates/lp-glsl/src/compiler.rs`

Similar changes needed in `compile_to_code_bytes()`.

### 4. Helper Function to Convert Function in Module

**File**: `crates/lp-glsl/src/transform/fixed32/mod.rs` (new function)

```rust
/// Convert a function in a module from float to fixed-point.
///
/// This extracts the function, converts it, and replaces it in the module.
pub fn convert_function_in_module(
    module: &mut dyn Module,
    func_id: FuncId,
    format: FixedPointFormat,
) -> Result<(), GlslError> {
    // 1. Get function signature from module
    // 2. Extract function body (this is tricky - functions are stored as Context)
    // 3. Convert function using rewrite_function()
    // 4. Replace function in module

    // Challenge: Module API doesn't expose function bodies directly
    // We may need to store Function objects separately and convert them
}
```

**Alternative**: Store user functions as `Function` objects during compilation, convert them all, then add to module.

### 5. Store User Functions During Compilation

**File**: `crates/lp-glsl/src/jit.rs`

Modify compilation flow:

```rust
struct CompiledUserFunction {
    name: String,
    func: Function,  // Store as Function object
    func_id: FuncId,
}

// During translate():
let mut compiled_user_funcs = Vec::new();

for user_func in &typed_ast.user_functions {
    // Compile function to Function object (don't add to module yet)
    let func = compile_user_function_to_function(...)?;

    // Convert to fixed-point if needed
    let func = if let Some(format) = self.fixed_point_format {
        crate::transform::fixed32::rewrite::rewrite_function(&func, format)?
    } else {
        func
    };

    // Now add to module with converted signature
    let func_id = self.module.declare_function(...)?;
    let mut ctx = Context::for_function(func);
    self.module.define_function(func_id, &mut ctx)?;

    compiled_user_funcs.push(CompiledUserFunction { name, func, func_id });
}
```

## Testing

### Test File: `crates/lp-glsl-filetests/filetests/fixed32/functions.glsl`

Create comprehensive tests for user functions:

```glsl
// test compile
// test fixed32
// test run

// Test 1: Simple user function with float parameters
float add(float a, float b) {
    return a + b;
}

float main() {
    return add(1.5, 2.5);  // Should be 4.0
}

// run: ~= 4.0

---

// Test 2: User function calling another user function
float multiply(float a, float b) {
    return a * b;
}

float square(float x) {
    return multiply(x, x);
}

float main() {
    return square(2.5);  // Should be 6.25
}

// run: ~= 6.25

---

// Test 3: User function with vector parameters
vec2 add_vec2(vec2 a, vec2 b) {
    return a + b;
}

float main() {
    vec2 a = vec2(1.5, 2.5);
    vec2 b = vec2(3.0, 4.0);
    vec2 result = add_vec2(a, b);
    return result.x + result.y;  // Should be 11.0
}

// run: ~= 11.0

---

// Test 4: Recursive function (if supported)
float factorial(float n) {
    if (n <= 1.0) {
        return 1.0;
    }
    return n * factorial(n - 1.0);
}

float main() {
    return factorial(5.0);  // Should be 120.0
}

// run: ~= 120.0
```

## Success Criteria

- [ ] User functions are converted to fixed-point when fixed-point mode is enabled
- [ ] Function calls between main and user functions work correctly
- [ ] Function signatures (parameters and returns) are converted correctly
- [ ] Tests in `functions.glsl` all pass
- [ ] No F32 types remain in user functions after conversion
- [ ] Function verification passes for converted user functions

## Files to Modify

1. `crates/lp-glsl/src/transform/fixed32/converters/calls.rs` - Detect internal calls
2. `crates/lp-glsl/src/jit.rs` - Convert user functions before main
3. `crates/lp-glsl/src/compiler.rs` - Convert user functions before main
4. `crates/lp-glsl/src/transform/fixed32/mod.rs` - Add helper functions
5. `crates/lp-glsl-filetests/filetests/fixed32/functions.glsl` - NEW test file

## Notes

- This stage focuses ONLY on user-defined functions, not intrinsics
- Intrinsics will be handled in later stages
- The approach may need refinement based on how Cranelift Module API works
- We may need to refactor how functions are stored/accessed in the module
