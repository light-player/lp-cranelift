# Analysis: Unsupported Features for riscv32

## Summary

After analyzing the codebase, here's what needs to be supported vs. what can be skipped:

## 1. StructReturn - **MUST BE SUPPORTED** ✅

### Why It's Critical

Light Player **actively uses StructReturn** for vector/matrix returns from GLSL shaders:

- **`lightplayer/crates/lp-glsl/src/backend/emu.rs`**: Uses StructReturn to return vector results from shader functions
- **`lightplayer/crates/lp-glsl/src/codegen/expr/function.rs`**: Sets up StructReturn buffers for function calls
- **`lightplayer/crates/lp-glsl/src/codegen/stmt.rs`**: Handles StructReturn in return statements
- **`lightplayer/crates/lp-jit-util/src/call.rs`**: Has `call_structreturn_riscv32()` implementation
- **`lightplayer/apps/test-structreturn/`**: Dedicated test application for StructReturn

### Current State

- ✅ StructReturn **calling** is implemented (`call_structreturn_riscv32`)
- ✅ StructReturn **parameter handling** exists in ABI code (`add_ret_area_ptr`)
- ❌ **Automatic conversion** to StructReturn when functions return >2 register values is missing

### The Problem

When a function returns more than 2 register values (e.g., `i64, i8` = 3 registers on riscv32), the compiler errors:

```
Unsupported feature: Too many return values to fit in registers. Use a StructReturn argument instead. (#9510)
```

Instead of automatically converting to StructReturn, it just errors out.

### What Needs to Be Done

1. **Automatic StructReturn conversion**: When a function returns >2 register values, automatically convert to StructReturn
2. **ABI support**: Ensure StructReturn is properly handled in `cranelift/codegen/src/isa/riscv32/abi.rs`
3. **Code generation**: Ensure return values are written to the StructReturn buffer

### Test Files Affected

- `stack.clif` - Actually tests stack operations, not StructReturn (may be false positive)
- `smul_overflow.clif` - Functions like `smulof_i64(i64, i64) -> i64, i8` return 3 registers
- `umul_overflow.clif` - Same issue with `umulof_i64(i64, i64) -> i64, i8`

## 2. Small Type Overflow Operations - **SHOULD BE SUPPORTED** ⚠️

### Why It Matters

While not directly used in Light Player's current codebase, these are standard CLIF operations that should be supported for:

- **Completeness**: Supporting all integer types (i8, i16, i32, i64)
- **Future compatibility**: May be needed for future features or optimizations
- **Test coverage**: Filetests expect these operations to work

### Current State

- ✅ `uadd_overflow` implemented for: i32, i64, i128
- ❌ `uadd_overflow` **NOT** implemented for: i8, i16
- ❌ `smul_overflow` **NOT** implemented at all
- ❌ `umul_overflow` **NOT** implemented at all
- ❌ Other overflow ops (`sadd_overflow`, `usub_overflow`, `ssub_overflow`) likely missing for small types

### The Problem

```
Unsupported feature: should be implemented in ISLE: inst = `v2, v3 = uadd_overflow.i8 v0, v1`, type = `Some(types::I8)`
```

### What Needs to Be Done

Add ISLE rules in `cranelift/codegen/src/isa/riscv32/lower.isle`:

1. **i8/i16 overflow operations**:

   - Zero-extend to i32
   - Perform operation at i32 width
   - Check for overflow (result doesn't fit in original width)
   - Extract result and overflow flag

2. **Multiply overflow operations**:
   - Implement `smul_overflow` and `umul_overflow` for all types
   - Use multi-instruction sequences for riscv32

### Test Files Affected

- `uadd_overflow_narrow.clif` - Tests i8/i16 overflow operations

## 3. f64 Operations - **SEPARATE ISSUE** (Not in original scope)

The plan mentions f64 operations, but this is a separate feature (requires D extension support).

## Recommendations

### Priority 1: StructReturn (CRITICAL)

**Must implement** - Light Player depends on this for vector/matrix returns.

**Implementation approach:**

1. Update ABI to automatically convert functions returning >2 register values to StructReturn
2. Ensure StructReturn buffer handling works correctly
3. Test with Light Player's actual use cases

### Priority 2: Small Type Overflow Operations (RECOMMENDED)

**Should implement** - Standard CLIF operations that should be supported.

**Implementation approach:**

1. Add ISLE rules for i8/i16 overflow operations
2. Add ISLE rules for multiply overflow operations
3. Follow patterns from other ISAs (aarch64, x64) for reference

### Priority 3: f64 Operations (FUTURE)

Can be deferred - requires D extension support and is a larger feature.

## Next Steps

1. **Implement StructReturn automatic conversion** (Priority 1)
2. **Implement small type overflow operations** (Priority 2)
3. **Update plan document** to reflect that these are required, not optional
