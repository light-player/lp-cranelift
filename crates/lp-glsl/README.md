# lp-glsl - GLSL Compiler Frontend (Phase 1)

A GLSL fragment shader compiler using Cranelift JIT for the Light Player project.

## Phase 1 Status

**Phase 1 is complete!** ✅

This phase establishes the complete architecture and implements basic functionality.

### Implemented Features

- ✅ Full module structure and architecture
- ✅ Type system (int/bool only in Phase 1)
- ✅ Symbol table and scope management
- ✅ Semantic analysis infrastructure
- ✅ Cranelift codegen infrastructure
- ✅ Basic operations: arithmetic (+, -, \*, /), comparisons (==, !=, <, >, <=, >=)
- ✅ Variable declarations and assignments
- ✅ Integer and boolean types
- ✅ Function with return value (int main() or bool main())
- ✅ JIT compilation and execution
- ✅ Runtime tests (16 tests, all passing)

### Not Yet Implemented (Future Phases)

- ❌ Control flow (if/else, loops)
- ❌ Floating-point operations
- ❌ Vectors, matrices
- ❌ Structs
- ❌ Function calls (user-defined or built-ins)
- ❌ Arrays
- ❌ Fixed-point transformation

## Usage

```rust
use lp_glsl::Compiler;

fn main() {
    let mut compiler = Compiler::new();

    // Compile GLSL shader that returns an integer
    let shader = r#"
        int main() {
            int a = 10;
            int b = 32;
            return a + b;
        }
    "#;

    // Compile to native code
    let func = compiler.compile_int(shader).unwrap();

    // Execute and get result
    let result = func();
    assert_eq!(result, 42);

    println!("Shader returned: {}", result);
}
```

## Examples

See `examples/simple.rs` for more examples including:
- Integer arithmetic
- Boolean comparisons
- Complex expressions

Run the example:
```bash
cargo run -p lp-glsl --example simple
```

## Testing

Phase 1 includes comprehensive runtime tests:

```bash
# Run all tests
cargo test -p lp-glsl

# Run integer tests only
cargo test -p lp-glsl --test runtime_int

# Run boolean tests only
cargo test -p lp-glsl --test runtime_bool
```

All 16 tests pass:
- 8 integer tests (literals, arithmetic, assignments)
- 8 boolean tests (literals, comparisons, NOT operator)

## Architecture

### Module Structure

- `lib.rs` - Main entry point, `#![no_std]` with std feature gate
- `frontend.rs` - GLSL parsing (re-export from glsl-parser)
- `semantic/` - Semantic analysis and type checking
  - `types.rs` - Type system
  - `scope.rs` - Symbol table and scope management
  - `mod.rs` - Semantic validation pass
- `codegen/` - Cranelift IR code generation
  - `context.rs` - Code generation context
  - `expr.rs` - Expression handlers
  - `stmt.rs` - Statement handlers
- `jit.rs` - JIT compilation interface
- `compiler.rs` - Main compiler orchestration

### Compilation Pipeline

```
GLSL Source
    ↓
[1] Parse (glsl-parser) → AST
    ↓
[2] Semantic Analysis → Typed AST
    ↓
[3] Cranelift Codegen → Cranelift IR
    ↓
[4] Cranelift Backend → Machine Code
    ↓
[5] JIT Execute → Runtime Validation
```

## Dependencies

- `glsl` - GLSL parser (from glsl-parser crate)
- `cranelift-frontend` - Frontend for Cranelift IR generation
- `cranelift-codegen` - Cranelift code generator
- `cranelift-jit` - JIT compiler (std feature only)
- `cranelift-module` - Module management
- `hashbrown` - Hash map implementation

## Features

- `std` (default) - Enable standard library support, includes JIT compilation
- `core` - Core functionality without std

## License

Apache-2.0 WITH LLVM-exception

