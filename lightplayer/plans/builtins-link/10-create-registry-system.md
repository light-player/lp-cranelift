# Phase 10: Create registry system

## Goal

Generate `BuiltinId` enum and registry for type-safe references and iteration for JIT linking.

## Steps

### 10.1 Design registry structure

- `BuiltinId` enum with variants for each builtin (e.g., `Fixed32Div`, `Fixed32Mul`, `Fixed32Sqrt`)
- `BuiltinRegistry` struct mapping IDs to function pointers/addresses
- Helper functions to iterate and link all builtins

### 10.2 Generate registry code

- Create code generation script or macro
- Extract builtin functions from `lp-builtins` (parse source or extract from ELF)
- Generate `BuiltinId` enum variants
- Generate registry initialization code

### 10.3 Implement registry

- Create `src/registry.rs` in `lp-glsl` or separate module
- Implement `BuiltinRegistry` with function pointer storage
- Implement iteration for linking all builtins

### 10.4 Use registry in JIT setup

- Replace manual function declarations with registry iteration
- Use registry to link all builtins automatically
- Ensure type safety (enum prevents typos, wrong function names)

## Registry Structure

```rust
pub enum BuiltinId {
    Fixed32Div,
    Fixed32Mul,
    Fixed32Sqrt,
    // ... more builtins
}

pub struct BuiltinRegistry {
    functions: HashMap<BuiltinId, FunctionPointer>,
}

impl BuiltinRegistry {
    pub fn link_all(&self, module: &mut Module) -> Result<()> {
        // Iterate and link all functions
    }
}
```

## Files to Create

- `lightplayer/crates/lp-glsl/src/backend/builtins/registry.rs` (or similar)
- Code generation script/macro for `BuiltinId` enum

## Files to Modify

- JIT setup code (use registry instead of manual declarations)

## Success Criteria

- `BuiltinId` enum exists with all builtin variants
- `BuiltinRegistry` can link all builtins automatically
- Type-safe references prevent errors
- Registry iteration works for JIT linking

## Notes

- Registry can be generated at build time (via `build.rs`) or compile time (via macro)
- Consider using `include!` or proc macro to generate from source
- Registry should make it easy to add new builtins (just add to enum)

