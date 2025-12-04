# GLSL Compiler Frontend Architecture

## Overview

Build a GLSL fragment shader compiler in `crates/lp-glsl` that parses GLSL, performs semantic analysis, generates Cranelift IR, and optionally transforms floating-point operations to fixed 16.16 integer math for RISC-V32 IMAC targets.

## Source Material References

This architecture draws from three main sources:

1. **DirectXShaderCompiler (DXC)** - `/Users/yona/dev/photomancer/DirectXShaderCompiler`

   - HLSL/GLSL compiler architecture patterns
   - Type system and semantic analysis
   - Expression/statement/declaration handlers

2. **rustc_codegen_cranelift** - `/Users/yona/dev/photomancer/rustc_codegen_cranelift`

   - Cranelift IR generation patterns
   - Intrinsics and built-in function handling
   - Value/place abstraction for code generation

3. **cranelift-examples** - `/Users/yona/dev/photomancer/cranelift-examples`
   - Struct lowering and type layout patterns
   - Function calling conventions
   - Stack allocation strategies

## Crate Structure

Create `crates/lp-glsl/` with the following module organization:

- **`lib.rs`** - Main entry point, `#![no_std]` with std feature gate

  - Reference: `lp-toy-lang/src/lib.rs`

- **`frontend/mod.rs`** - GLSL parsing (re-export from glsl-parser)

  - Uses: `/Users/yona/dev/photomancer/glsl-parser/glsl` crate
  - Parse to `glsl::syntax::TranslationUnit`

- **`ast.rs`** - Lightweight wrapper/adapter around glsl-parser AST

  - Reference: DXC's AST handling in `tools/clang/lib/AST/`

- **`semantic/mod.rs`** - Semantic analysis and type checking

  - `semantic/types.rs` - Type system (vec2/vec3/vec4, mat2/mat3/mat4, float, int, bool, samplers, structs)
    - Reference: DXC `tools/clang/lib/AST/Type.cpp`, `HlslTypes.cpp`
  - `semantic/scope.rs` - Symbol table and scope management
    - Reference: DXC `tools/clang/lib/SPIRV/DeclResultIdMapper.h` (lines 1-150)
  - `semantic/builtins.rs` - Built-in functions registry (sin, cos, texture2D, etc.)
    - Reference: rustc_codegen_cranelift `src/intrinsics/mod.rs`
  - `semantic/validator.rs` - Semantic validation pass
    - Reference: DXC semantic analysis patterns

- **`ir/mod.rs`** - Intermediate representation layer

  - `ir/typed_ast.rs` - Type-annotated AST after semantic analysis
  - `ir/lowering.rs` - Lower typed AST to Cranelift-friendly form
    - Reference: cranelift-examples `examples/lowering-structs/lower.rs`

- **`codegen/mod.rs`** - Cranelift IR code generation

  - `codegen/translator.rs` - Main AST-to-Cranelift translator
    - Reference: DXC `tools/clang/lib/SPIRV/SpirvEmitter.h` structure
    - Reference: rustc_codegen_cranelift `src/base.rs` for FunctionCx pattern
  - `codegen/builtins.rs` - Built-in function implementations
    - Reference: rustc_codegen_cranelift `src/intrinsics/mod.rs`
  - `codegen/context.rs` - Code generation context (variables, blocks, etc.)
    - Reference: `lp-toy-lang/src/jit.rs` FunctionTranslator pattern

- **`fixed_point/mod.rs`** - Fixed-point transformation (optional pass)

  - `fixed_point/transform.rs` - Cranelift IR pass to convert f32 -> i32 (16.16 fixed)
  - `fixed_point/runtime.rs` - Fixed-point math runtime functions

- **`jit.rs`** - JIT compilation interface

  - Reference: `lp-toy-lang/src/jit.rs` (lines 1-250)

- **`compiler.rs`** - Main compiler orchestration
  - Reference: DXC `tools/clang/lib/SPIRV/EmitSpirvAction.cpp`

## Compilation Pipeline

```
GLSL Source
    ↓
[1] Parse (glsl-parser) → AST
    ↓
[2] Semantic Analysis → Typed AST
    ↓
[3] IR Lowering → Simplified IR
    ↓
[4] Cranelift Codegen → Cranelift IR (with float ops)
    ↓
[5] [OPTIONAL] Fixed-Point Transform → Cranelift IR (int ops only)
    ↓
[6] Cranelift Backend → RISC-V32 Machine Code
```

## Phase Details

### Phase 1: Parsing

- Use `/Users/yona/dev/photomancer/glsl-parser` (vendored or as path dependency)
- Parse to `glsl::syntax::TranslationUnit`
- Create thin wrapper types in `crates/lp-glsl/src/ast.rs` if needed for easier handling
- **Pattern**: Similar to DXC's AST consumption in `SpirvEmitter::HandleTranslationUnit`

### Phase 2: Semantic Analysis

**Reference**: DXC `tools/clang/lib/SPIRV/DeclResultIdMapper.{h,cpp}` for symbol management

- **Type checking**: Validate all expressions, handle vector/matrix math
  - Follow DXC pattern from `tools/clang/lib/AST/Type.cpp`
  - GLSL type mappings in `HlslTypes.cpp` as reference
- **Symbol resolution**: Build symbol tables for uniforms, varyings, locals, functions
  - Reference: `DeclResultIdMapper` class (lines 30-200) for resource variable tracking
  - Use `ResourceVar` pattern for uniforms/samplers (lines 35-73)
  - Use `CounterVarFields` pattern (lines 103-150) for complex struct hierarchies
- **Built-in registry**: Map GLSL built-ins (texture2D, sin, cos, dot, cross, etc.)
  - Reference: rustc_codegen_cranelift `src/intrinsics/mod.rs` (lines 1-150)
  - Pattern: Registry with multiple signatures per function name
- **Scope analysis**: Handle function scopes, block scopes, global scope
  - Stack-based scope management
- **Output**: Typed AST where every expression has resolved type information

Key challenges:

- **Structs**: Track struct definitions, member access
  - Reference: cranelift-examples `examples/lowering-structs/` for layout calculation
  - Use `CounterVarFields` pattern from DXC for nested struct handling
- **Swizzling**: vec3.xyz, vec4.xxyz - validate and type correctly
  - Reference: DXC `SpirvEmitter.h` line 168 `doHLSLVectorElementExpr`
- **Function overloading**: Built-ins have multiple signatures (texture2D variants)
  - Reference: rustc_codegen_cranelift intrinsics dispatch pattern
- **Inout parameters**: Track mutable references through function calls
  - Reference: DXC handling of inout/out parameters in function calls

### Phase 3: IR Lowering

**Reference**: cranelift-examples `examples/lowering-structs/lower.rs` (lines 1-377)

- Desugar complex GLSL constructs:
  - Vector/matrix operations → scalar operations
    - Use `simd_for_each_lane` pattern from rustc_codegen_cranelift (lines 67-89)
  - Swizzling → extract/insert operations
    - Reference: DXC `doHLSLVectorElementExpr` pattern
  - Constructor calls → component assignments
    - Use `construct_struct` pattern (lines 192-210)
- Flatten nested expressions for easier codegen
- Resolve all function calls (inline built-ins or prepare for libcalls)
  - Reference: `call_func` from cranelift-examples (lines 149-185)

### Phase 4: Cranelift Code Generation

**Reference**: Multiple sources

- Use `cranelift_frontend::FunctionBuilder` pattern
  - Main reference: rustc_codegen_cranelift `src/base.rs` (lines 60-147)
  - Structure: `codegen_fn` → `FunctionCx` → `codegen_fn_body`
- **Expression handling** - Pattern from DXC `SpirvEmitter.h`:
  - `doExpr()` dispatcher (line 82)
  - `doBinaryOperator()` (line 153)
  - `doCallExpr()` (line 154)
  - `doConditionalOperator()` (lines 159-163)
  - `doUnaryOperator()` (line 174)
- **Statement handling** - Pattern from DXC `SpirvEmitter.h`:

  - `doStmt()` dispatcher (line 81)
  - `doForStmt()` (line 142)
  - `doIfStmt()` (line 143)
  - `doWhileStmt()` / `doDoStmt()` (lines 147-148)
  - `doReturnStmt()` (line 144)
  - `doSwitchStmt()` (line 145)

- Map GLSL types to Cranelift types:
  - `float` → `types::F32`
  - `vec2/3/4` → Use `VirtualValue` pattern from cranelift-examples (lines 1-99)
    - Option 1: Struct with scalar fields (for register passing)
    - Option 2: Stack allocation with pointer (for large types)
  - `mat4` → struct/array with 16× F32 (column-major, GLSL convention)
  - `int` → `types::I32`
  - `bool` → `types::I8` (1 byte) or `types::I32`
- **Type layout calculation**:
  - Reference: cranelift-examples `examples/struct-layouts/main.rs`
  - Alignment and padding for GLSL std140/std430 layouts
- Implement built-in functions:
  - Math: Generate Cranelift libcalls (sin, cos, sqrt, pow)
    - Pattern: rustc_codegen_cranelift intrinsics dispatch (lines 100-150)
  - Vector ops: Expand to component-wise operations
    - Use `simd_pair_for_each_lane` pattern (lines 115-140)
  - Texture sampling: External function calls (provided by runtime)
    - Use `call_func` pattern from cranelift-examples

### Phase 5: Fixed-Point Transform (Optional)

- **When**: After Cranelift IR generation, before final compilation
- **How**: Cranelift IR pass that walks instructions:
  - Replace F32 types with I32
  - Replace `fadd` → fixed_add (multiply by 2^16, add, shift)
  - Replace `fmul` → fixed_mul (multiply, shift right 16)
  - Replace `fdiv` → fixed_div (shift left 16, divide)
  - Replace `fsqrt`, `fsin`, `fcos` → libcall to fixed-point implementations
- **Runtime library**: Provide fixed-point math functions in `fixed_point/runtime.rs`
- **Precision**: 16.16 fixed point (16 bits integer, 16 bits fractional)

### Phase 6: Machine Code Generation

- Target: RISC-V32 IMAC (no F extension)
- Use Cranelift's RISC-V backend
- Handle ABIs for uniforms, varyings, output

## Data Structures

### TypedExpr

**Reference**: DXC type-annotated expression handling

```rust
struct TypedExpr {
    expr: Expr,           // Original AST node (from glsl-parser)
    ty: Type,             // Resolved type
    location: SourceLoc, // For error reporting
}
```

### Type System

**Reference**: DXC `tools/clang/lib/AST/Type.cpp` and `HlslTypes.cpp`

```rust
enum Type {
    Void,
    Bool,
    Int,
    Float,
    Vec2, Vec3, Vec4,      // Vector types
    IVec2, IVec3, IVec4,   // Integer vector types
    BVec2, BVec3, BVec4,   // Boolean vector types
    Mat2, Mat3, Mat4,      // Matrix types (float only in GLSL ES)
    Sampler2D,             // Texture sampler types
    SamplerCube,
    Struct(StructId),      // User-defined structs
    Array(Box<Type>, usize),
}

struct StructDef {
    name: String,
    fields: Vec<FieldDef>,
    size: u32,             // Total size in bytes
    alignment: u32,        // Alignment requirement
}

struct FieldDef {
    name: String,
    ty: Type,
    offset: u32,           // Byte offset from struct start
}
```

### Symbol Table

**Reference**: DXC `DeclResultIdMapper.h` (lines 30-200)

```rust
struct SymbolTable {
    scopes: Vec<Scope>,    // Stack of scopes (function, block, global)
    functions: HashMap<String, FunctionSig>,
    structs: HashMap<String, StructDef>,
    builtins: BuiltinRegistry,
}

struct Scope {
    variables: HashMap<String, VarDecl>,
    parent: Option<ScopeId>,
}

// Reference: ResourceVar pattern from DXC (lines 35-73)
struct VarDecl {
    name: String,
    ty: Type,
    storage_class: StorageClass,  // uniform, varying, local
    location: Option<u32>,        // location qualifier
}

enum StorageClass {
    Uniform,      // Shader uniforms
    In,           // Input varyings (from vertex shader)
    Out,          // Output varyings (to next stage)
    Local,        // Function-local variables
}
```

### Built-in Function Registry

**Reference**: rustc_codegen_cranelift `src/intrinsics/mod.rs`

```rust
struct BuiltinRegistry {
    math_fns: HashMap<String, Vec<BuiltinSig>>,     // sin, cos, sqrt, pow, etc.
    vector_fns: HashMap<String, Vec<BuiltinSig>>,   // dot, cross, length, normalize
    texture_fns: HashMap<String, Vec<BuiltinSig>>,  // texture2D, textureCube
    matrix_fns: HashMap<String, Vec<BuiltinSig>>,   // matrixCompMult
}

struct BuiltinSig {
    name: String,
    params: Vec<Type>,
    return_type: Type,
    implementation: BuiltinImpl,
}

enum BuiltinImpl {
    Libcall(String),           // External function call
    Intrinsic(IntrinsicKind),  // Inline Cranelift IR generation
    ComponentWise(BinaryOp),   // Expand to per-component operations
}
```

### Code Generation Context

**Reference**: rustc_codegen_cranelift `src/base.rs` FunctionCx (lines 94-118)

```rust
struct CodegenContext<'a> {
    builder: FunctionBuilder<'a>,
    module: &'a mut dyn Module,

    // Symbol mapping
    variables: HashMap<String, Variable>,  // Cranelift Variable for each GLSL variable

    // Type information
    pointer_type: Type,                    // Target pointer size

    // Current function context
    return_type: Type,

    // Blocks for control flow
    current_block: Option<Block>,
    loop_exit: Option<Block>,              // For break statements
    loop_header: Option<Block>,            // For continue statements
}
```

### VirtualValue Pattern (for vectors/structs)

**Reference**: cranelift-examples `examples/lowering-structs/lower.rs` (lines 1-99)

```rust
// Abstraction over Cranelift values to handle aggregates
enum VirtualValue {
    Scalar(Value),                         // Single Cranelift value (int, float, bool)
    StackStruct {                          // Struct allocated on stack
        type_name: String,
        ptr: Value,                        // Pointer to struct
    },
    RegisterStruct {                       // Struct passed in registers
        type_name: String,
        fields: Vec<VirtualValue>,         // Individual field values
    },
}
```

## Expression and Statement Handler Structure

**Reference**: DXC `tools/clang/lib/SPIRV/SpirvEmitter.h` (lines 80-176)

Model the code generator after DXC's organization:

### Main Dispatcher Pattern

```rust
// Reference: SpirvEmitter.h lines 80-82
impl<'a> CodegenContext<'a> {
    fn translate_expr(&mut self, expr: &Expr) -> VirtualValue {
        match expr {
            Expr::Binary(op, lhs, rhs) => self.do_binary_operator(op, lhs, rhs),
            Expr::Unary(op, expr) => self.do_unary_operator(op, expr),
            Expr::Call(name, args) => self.do_call_expr(name, args),
            Expr::Conditional(cond, then_e, else_e) => self.do_conditional(cond, then_e, else_e),
            Expr::Assignment(lhs, rhs) => self.do_assignment(lhs, rhs),
            Expr::Variable(name) => self.do_variable_ref(name),
            Expr::Dot(expr, field) => self.do_member_access(expr, field),
            Expr::Bracket(expr, idx) => self.do_array_subscript(expr, idx),
            // ... etc
        }
    }

    fn translate_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::If(cond, then_block, else_block) => self.do_if_stmt(cond, then_block, else_block),
            Stmt::While(cond, body) => self.do_while_stmt(cond, body),
            Stmt::For(init, cond, update, body) => self.do_for_stmt(init, cond, update, body),
            Stmt::Return(expr) => self.do_return_stmt(expr),
            Stmt::Break => self.do_break_stmt(),
            Stmt::Continue => self.do_continue_stmt(),
            Stmt::Discard => self.do_discard_stmt(),  // Fragment shader only
            // ... etc
        }
    }
}
```

### Expression Handlers

Reference: DXC `SpirvEmitter.h` lines 151-176

```rust
// Binary operators: +, -, *, /, %, ==, !=, <, >, etc.
fn do_binary_operator(&mut self, op: BinaryOp, lhs: &Expr, rhs: &Expr) -> VirtualValue {
    // Reference: SpirvEmitter.h line 153
    // Handle type conversions, vector operations, matrix operations
}

// Function calls (built-ins and user-defined)
fn do_call_expr(&mut self, name: &str, args: &[Expr]) -> VirtualValue {
    // Reference: SpirvEmitter.h line 154
    // Dispatch to built-in registry or user function
}

// Conditional operator: condition ? true_expr : false_expr
fn do_conditional(&mut self, cond: &Expr, true_e: &Expr, false_e: &Expr) -> VirtualValue {
    // Reference: SpirvEmitter.h lines 159-163
    // Create blocks for each branch, merge with phi node
}

// Member access: struct.field or vector.swizzle
fn do_member_access(&mut self, expr: &Expr, field: &str) -> VirtualValue {
    // Reference: SpirvEmitter.h line 172 (doMemberExpr)
    // Handle both struct fields and vector swizzles
}

// Vector swizzle: vec.xyzw, vec.rgba
fn do_vector_swizzle(&mut self, vec: VirtualValue, swizzle: &str) -> VirtualValue {
    // Reference: SpirvEmitter.h line 168 (doHLSLVectorElementExpr)
    // Extract and reorder components
}

// Array subscript: array[index]
fn do_array_subscript(&mut self, array: &Expr, index: &Expr) -> VirtualValue {
    // Reference: SpirvEmitter.h line 151 (doArraySubscriptExpr)
}
```

### Statement Handlers

Reference: DXC `SpirvEmitter.h` lines 139-149

```rust
// If statement
fn do_if_stmt(&mut self, cond: &Expr, then_block: &[Stmt], else_block: Option<&[Stmt]>) {
    // Reference: SpirvEmitter.h line 143
    // Create then/else/merge blocks, conditional branch
    // Pattern from lp-toy-lang/src/jit.rs lines 333-392
}

// For loop
fn do_for_stmt(&mut self, init: &Stmt, cond: &Expr, update: &Stmt, body: &[Stmt]) {
    // Reference: SpirvEmitter.h line 142
    // Header block, body block, continue block, exit block
}

// While loop
fn do_while_stmt(&mut self, cond: &Expr, body: &[Stmt]) {
    // Reference: SpirvEmitter.h line 147
    // Pattern from lp-toy-lang/src/jit.rs lines 394-424
}

// Return statement
fn do_return_stmt(&mut self, expr: Option<&Expr>) {
    // Reference: SpirvEmitter.h line 144
    // Evaluate expression, emit return instruction
}

// Switch statement
fn do_switch_stmt(&mut self, expr: &Expr, cases: &[SwitchCase]) {
    // Reference: SpirvEmitter.h line 145
    // Create block for each case, use br_table or cascading branches
}
```

## Testing Strategy

### Filetests (Cranelift-style)

Create `crates/lp-glsl/filetests/` directory:

```
filetests/
├── basic/
│   ├── literals.glsl
│   ├── arithmetic.glsl
│   └── variables.glsl
├── vectors/
│   ├── swizzle.glsl
│   ├── dot_product.glsl
│   └── constructors.glsl
├── functions/
│   ├── basic_call.glsl
│   ├── builtin_sin.glsl
│   └── user_defined.glsl
├── control_flow/
│   ├── if_else.glsl
│   ├── for_loop.glsl
│   └── while_loop.glsl
├── structs/
│   ├── definition.glsl
│   ├── member_access.glsl
│   └── nested.glsl
├── advanced/
│   ├── texture_sample.glsl
│   ├── matrix_multiply.glsl
│   └── inout_params.glsl
└── fixed_point/
    ├── fp_arithmetic.glsl
    └── fp_builtin.glsl
```

Each test file format:

```glsl
; Test: Basic addition
; CHECK: function u0:0(f32, f32) -> f32

void main() {
  float a = 1.0;
  float b = 2.0;
  float c = a + b;
}

; CHECK: fadd
```

Use test harness similar to Cranelift's filetest infrastructure in `cranelift/filetests/`.

### Unit Tests

- `semantic/` - Type checking, scope resolution
- `codegen/` - Individual construct translation
- `fixed_point/` - Fixed-point math accuracy

### Integration Tests

- End-to-end shader compilation in `tests/`
- Known-good GLSL shaders from reference implementations
- Compare output with reference compilers (for IR validation)

## Dependencies

Update `crates/lp-glsl/Cargo.toml`:

```toml
[dependencies]
glsl = { path = "/Users/yona/dev/photomancer/glsl-parser/glsl", default-features = false, features = ["alloc"] }
cranelift-frontend = { workspace = true }
cranelift-codegen = { workspace = true }
cranelift-jit = { workspace = true, optional = true }
cranelift-module = { workspace = true }
hashbrown = { workspace = true }  # For no_std HashMap

[features]
default = ["std"]
std = ["glsl/std", "cranelift-codegen/std", "cranelift-jit"]
core = ["cranelift-codegen/core"]
fixed-point = []  # Enable fixed-point transformation
```

## Implementation Stages

### Stage 1: Foundation

**Focus**: Basic infrastructure and parsing

Files to create:

- `crates/lp-glsl/Cargo.toml` - Dependencies and features
  - Reference: `lp-toy-lang/Cargo.toml`
- `crates/lp-glsl/src/lib.rs` - Module structure with `#![no_std]`
  - Reference: `lp-toy-lang/src/lib.rs`
- `crates/lp-glsl/src/frontend.rs` - Re-export glsl-parser
- `crates/lp-glsl/src/ast.rs` - Wrapper types (if needed)

Tasks:

- Set up crate with no_std + std features
- Add glsl-parser as path dependency
- Parse basic GLSL fragment shader to TranslationUnit
- Create simple test that parses a shader

### Stage 2: Core Semantics

**Focus**: Type system and symbol tables

Files to create:

- `crates/lp-glsl/src/semantic/types.rs`
  - Reference: DXC `tools/clang/lib/AST/Type.cpp`
- `crates/lp-glsl/src/semantic/scope.rs`
  - Reference: DXC `DeclResultIdMapper.h` ResourceVar pattern
- `crates/lp-glsl/src/semantic/builtins.rs`
  - Reference: rustc_codegen_cranelift `src/intrinsics/mod.rs`
- `crates/lp-glsl/src/semantic/validator.rs`

Tasks:

- Implement Type enum and StructDef
- Build symbol table with scope management
- Create built-in function registry (subset: sin, cos, dot, length)
- Type checking for basic expressions (arithmetic, comparisons)
- Validate variable references

### Stage 3: Basic Codegen

**Focus**: Simple expressions and function skeleton

Files to create:

- `crates/lp-glsl/src/codegen/context.rs`
  - Reference: rustc_codegen_cranelift `src/base.rs` FunctionCx
- `crates/lp-glsl/src/codegen/translator.rs`
  - Reference: DXC `SpirvEmitter.h` dispatcher pattern
- `crates/lp-glsl/src/jit.rs`
  - Reference: `lp-toy-lang/src/jit.rs`

Tasks:

- Implement CodegenContext with FunctionBuilder
- Translate scalar expressions (literals, variables, arithmetic)
- Handle assignments and variable declarations
- Implement basic function main() compilation
- Generate CLIF IR output for inspection

Test case:

```glsl
void main() {
    float x = 1.0;
    float y = 2.0;
    float z = x + y;
}
```

### Stage 4: Vectors & Matrices

**Focus**: Aggregate types and vector operations

Files to create:

- `crates/lp-glsl/src/codegen/vectors.rs`
  - Use VirtualValue pattern from cranelift-examples
- `crates/lp-glsl/src/ir/lowering.rs`
  - Reference: cranelift-examples `lowering-structs/lower.rs`

Tasks:

- Implement VirtualValue abstraction
- Vector construction: `vec3(1.0, 2.0, 3.0)`
- Vector swizzling: `vec.xyz`, `vec.xxy`
  - Reference: DXC doHLSLVectorElementExpr
- Component-wise operations: `vec3 + vec3`
  - Use simd_pair_for_each_lane pattern
- Vector built-ins: dot, cross, length, normalize
  - Reference: rustc_codegen_cranelift intrinsics
- Matrix types and operations

Test case:

```glsl
void main() {
    vec3 a = vec3(1.0, 2.0, 3.0);
    vec3 b = vec3(4.0, 5.0, 6.0);
    float d = dot(a, b);
    vec3 c = a.xzy + b;
}
```

### Stage 5: Control Flow

**Focus**: Conditionals and loops

Tasks:

- If/else statements
  - Reference: lp-toy-lang/src/jit.rs lines 333-392
- While loops
  - Reference: lp-toy-lang/src/jit.rs lines 394-424
- For loops
  - Reference: DXC doForStmt pattern
- Break/continue statements
  - Track loop_exit and loop_header blocks
- Early return handling

Test case:

```glsl
void main() {
    float sum = 0.0;
    for (int i = 0; i < 10; i++) {
        if (i % 2 == 0) {
            sum += float(i);
        }
    }
}
```

### Stage 6: Advanced Features

**Focus**: Structs, functions, and complex features

Tasks:

- User-defined structs
  - Reference: cranelift-examples `lowering-structs/`
  - Implement StructDef with layout calculation
  - Member access with offset calculation
- User-defined functions
  - Function declarations and calls
  - Parameter passing
- Inout parameters
  - Pass by pointer pattern
  - Load before, store after
- Texture sampling (stub external functions)

Test case:

```glsl
struct Light {
    vec3 position;
    vec3 color;
};

void applyLight(inout vec3 color, Light light) {
    color += light.color;
}

void main() {
    Light l;
    l.position = vec3(1.0, 2.0, 3.0);
    l.color = vec3(1.0, 0.5, 0.3);
    vec3 result = vec3(0.0);
    applyLight(result, l);
}
```

### Stage 7: Fixed-Point Transform

**Focus**: Post-compilation float-to-int transformation

Files to create:

- `crates/lp-glsl/src/fixed_point/transform.rs` - IR pass
- `crates/lp-glsl/src/fixed_point/runtime.rs` - Runtime functions
- `crates/lp-glsl/src/fixed_point/mod.rs`

Tasks:

- Walk Cranelift IR functions
- Identify and replace F32 types with I32
- Replace float operations:
  - `fadd(a, b)` → `iadd(a, b)` (16.16 fixed adds directly)
  - `fmul(a, b)` → `imul(a, b)` then `sshr(result, 16)`
  - `fdiv(a, b)` → `ishl(a, 16)` then `sdiv(a, b)`
- Replace float constants:
  - `1.0` → `0x00010000` (1 << 16)
  - `0.5` → `0x00008000` (1 << 15)
- Implement fixed-point runtime functions:
  - `__fp_sin`, `__fp_cos`, `__fp_sqrt`
  - Use lookup tables or CORDIC algorithms
- Add feature flag `fixed-point` to enable

Reference: RISC-V soft-float emulation patterns

### Stage 8: RISC-V Integration

**Focus**: End-to-end compilation and execution

Tasks:

- Configure Cranelift for RISC-V32 IMAC target
- Wire up to lp-riscv-tools emulator
  - Reference: `crates/lp-riscv-tools/src/emu/`
- Create test harness:
  - Compile GLSL shader
  - Apply fixed-point transform
  - Generate RISC-V32 machine code
  - Execute in emulator
  - Validate outputs
- Benchmark fixed-point precision vs float
- Optimize hot paths (vector ops, math functions)

Test: Full fragment shader

```glsl
uniform vec3 lightPos;
varying vec3 fragPos;
varying vec3 fragNormal;

void main() {
    vec3 lightDir = normalize(lightPos - fragPos);
    float diff = max(dot(fragNormal, lightDir), 0.0);
    vec3 color = vec3(1.0, 0.5, 0.3) * diff;
    gl_FragColor = vec4(color, 1.0);
}
```

## Hard Features to Consider

### Swizzling

**Reference**: DXC `SpirvEmitter.h` line 168 `doHLSLVectorElementExpr`

Parse: `vec3.xyz` → AST node with vector + swizzle mask

- glsl-parser provides this in the AST already
- Swizzle can be read (`vec3.xyz`) or write (`vec3.xy = vec2(1.0, 2.0)`)

Codegen approach:

1. **Read swizzle**: Extract components, reassemble into new vector

   ```rust
   // vec3.xzy → new vec3(original.x, original.z, original.y)
   let x = builder.ins().extractlane(vec, 0);
   let z = builder.ins().extractlane(vec, 2);
   let y = builder.ins().extractlane(vec, 1);
   let result = construct_vector([x, z, y]);
   ```

2. **Write swizzle**: Extract, modify components, reassemble
   - Requires tracking lvalue vs rvalue expressions
   - Reference: DXC's `loadIfGLValue` pattern (line 95)

### Built-in Functions

**Reference**: rustc_codegen_cranelift `src/intrinsics/mod.rs`

Maintain registry with type signatures and implementations

- Multiple signatures per function (overloading)
- Dispatch based on argument types

Examples:

- `sin(float)` → libcall to `sinf`
  - Reference: rustc intrinsics libcall pattern
- `dot(vec3, vec3)` → expand to `a.x*b.x + a.y*b.y + a.z*b.z`
  - Use `simd_pair_for_each_lane` pattern (lines 115-140)
- `texture2D(sampler2D, vec2)` → external function call
  - Sampler is opaque handle, actual sampling done by runtime

Built-in categories:

1. **Math intrinsics** (sin, cos, sqrt, pow, exp, log):
   - Libcalls to standard math library
   - Will need fixed-point implementations for RISC-V
2. **Vector intrinsics** (dot, cross, length, normalize, reflect):
   - Expand to component-wise operations
   - Some can optimize to SIMD on targets that support it
3. **Matrix intrinsics** (matrixCompMult, transpose):
   - Expand to element-wise operations
4. **Texture intrinsics** (texture2D, textureCube):
   - External function calls to runtime

### Inout Parameters

**Reference**: DXC function call handling

Semantic analysis:

- Track which parameters are `in`, `out`, `inout`
- Validate that `out`/`inout` parameters are lvalues (assignable)

Codegen:

- `in`: Pass by value (copy)
- `out`: Pass pointer, only write back
- `inout`: Pass pointer, read before and write after

  ```rust
  // void foo(inout vec3 v) { v.x += 1.0; }
  // Calling: foo(my_vec);

  let ptr = address_of(my_vec);  // Get pointer
  call(foo, [ptr]);              // Pass pointer
  // Function modifies through pointer
  ```

### Structs

**Reference**: cranelift-examples `examples/lowering-structs/`

Symbol table:

- Track struct definitions with field names, types, offsets
- Reference: `StructDef` data structure above

Codegen strategies (from cranelift-examples lines 64-99):

1. **Small structs** (fit in registers):
   - Pass as multiple Cranelift values
   - `struct Foo { float x, y; }` → pass as `(f32, f32)`
2. **Large structs** (don't fit in registers):
   - Pass by pointer
   - Allocate on stack with `stack_alloc_struct` (lines 365-375)
3. **Member access**:
   - Calculate offset: `base_ptr + field_offset`
   - Load/store at offset
   - Reference: `destruct_field` (lines 212-239)
4. **Nested structs**:
   - Flatten offsets recursively
   - Reference: `deref_fields` (lines 288-311)

### Texture Sampling

**Reference**: DXC external function calls

Codegen:

- Declare external function signature in Cranelift
- Sampler2D is opaque handle (integer or pointer)
- Runtime provides implementation

```rust
// GLSL: vec4 color = texture2D(tex, uv);
// Cranelift:
let tex_handle = load_variable("tex");   // Get sampler handle
let uv_x = load_variable("uv").extract_x();
let uv_y = load_variable("uv").extract_y();

let tex_fn = declare_import("__glsl_texture2D",
    signature([I32, F32, F32], [F32, F32, F32, F32]));
let result = call(tex_fn, [tex_handle, uv_x, uv_y]);
```

Runtime must provide:

- `__glsl_texture2D(sampler, u, v) -> (r, g, b, a)`
- Other texture functions as needed

### Matrix Math

**Reference**: DXC matrix handling

Storage: Column-major arrays (GLSL convention)

- `mat4` = 4 columns × 4 rows = 16 floats
- `mat[i]` accesses column `i` (returns vec4)
- `mat[i][j]` accesses element at column `i`, row `j`

Operations:

1. **Matrix construction**:

   ```glsl
   mat3 m = mat3(1.0);  // Identity matrix
   mat3 m = mat3(v1, v2, v3);  // From column vectors
   ```

2. **Matrix multiplication** (`mat * mat`):
   - Expand to vector operations (dot products)
   - Or call optimized function (BLAS-style)
3. **Matrix-vector multiplication** (`mat * vec`):

   ```glsl
   vec3 result = mat3_val * vec3_val;
   // result.x = dot(mat3_val[0], vec3_val);
   // result.y = dot(mat3_val[1], vec3_val);
   // result.z = dot(mat3_val[2], vec3_val);
   ```

4. **Component-wise operations** (`matrixCompMult`):
   - Iterate over all elements
   - Apply operation element-wise

## Quick Reference: Where to Look

When implementing a specific feature, reference these files:

### Parser Integration

- `/Users/yona/dev/photomancer/glsl-parser/glsl/src/lib.rs` - Main parser API
- `/Users/yona/dev/photomancer/glsl-parser/glsl/src/syntax.rs` - AST definitions
- `/Users/yona/dev/photomancer/glsl-parser/glsl/src/visitor.rs` - AST traversal

### Type System and Semantics

- DXC `tools/clang/lib/AST/Type.cpp` - Type representations
- DXC `tools/clang/lib/AST/HlslTypes.cpp` - HLSL-specific types
- DXC `tools/clang/lib/SPIRV/DeclResultIdMapper.h` - Symbol table patterns

### Expression Codegen

- DXC `tools/clang/lib/SPIRV/SpirvEmitter.h` (lines 151-176) - Expression handlers
- DXC `tools/clang/lib/CodeGen/CGExpr.cpp` - Expression lowering examples
- `lp-toy-lang/src/jit.rs` (lines 260-315) - Basic expression translation

### Statement Codegen

- DXC `tools/clang/lib/SPIRV/SpirvEmitter.h` (lines 139-149) - Statement handlers
- `lp-toy-lang/src/jit.rs` (lines 333-424) - If/else and loops

### Built-in Functions

- rustc_codegen_cranelift `src/intrinsics/mod.rs` - Intrinsics dispatch
- rustc_codegen_cranelift `src/intrinsics/simd.rs` - SIMD operations
- rustc_codegen_cranelift `src/intrinsics/llvm.rs` - Math libcalls

### Vector Operations

- rustc_codegen_cranelift `src/intrinsics/mod.rs` (lines 67-140) - SIMD patterns
- DXC `tools/clang/lib/SPIRV/SpirvEmitter.h` line 168 - Vector element access

### Struct Handling

- cranelift-examples `examples/lowering-structs/lower.rs` - Complete struct lowering
- cranelift-examples `examples/lowering-structs/types.rs` - Type layouts
- cranelift-examples `examples/struct-layouts/main.rs` - Layout calculation

### Function Calls and ABI

- cranelift-examples `examples/lowering-structs/lower.rs` (lines 149-185) - Call handling
- rustc_codegen_cranelift `src/abi/mod.rs` - ABI handling patterns

### Control Flow

- `lp-toy-lang/src/jit.rs` (lines 333-424) - If/else and loops
- rustc_codegen_cranelift `src/base.rs` - Complex control flow

### Fixed-Point Math

- Look for RISC-V soft-float implementations
- CORDIC algorithm references for sin/cos
- Fixed-point multiply/divide patterns

## File Locations

### Main Crate

- `crates/lp-glsl/` - Main compiler crate
  - `src/lib.rs` - Public API
  - `src/frontend.rs` - Parser integration
  - `src/semantic/` - Type checking and validation
  - `src/ir/` - IR lowering
  - `src/codegen/` - Cranelift code generation
  - `src/fixed_point/` - Fixed-point transformation
  - `src/jit.rs` - JIT interface

### Testing

- `crates/lp-glsl/filetests/` - Cranelift-style filetests
  - `basic/` - Literals, arithmetic, variables
  - `vectors/` - Vector operations and swizzling
  - `functions/` - Built-ins and user functions
  - `control_flow/` - If/else, loops
  - `structs/` - Struct definition and usage
  - `advanced/` - Complex features
  - `fixed_point/` - Fixed-point tests
- `crates/lp-glsl/src/*/tests.rs` - Unit tests per module
- `crates/lp-glsl/tests/` - Integration tests
  - `compile_shader.rs` - End-to-end compilation
  - `execute_shader.rs` - Execution tests with RISC-V emu

### Examples

- `apps/lp-glsl-example/` - Simple shader JIT demo
  - `src/main.rs` - Compile and run a basic fragment shader
  - `shaders/` - Example GLSL shaders

## Key Architectural Patterns

Drawing from the three source materials:

### 1. Visitor/Dispatcher Pattern (from DXC)

**Pattern**: Single dispatch point for each AST node type

- `translate_expr(&Expr)` dispatches to specific handlers
- `translate_stmt(&Stmt)` dispatches to specific handlers
- Each handler is a focused function doing one thing

**Benefit**: Clear separation of concerns, easy to test individual constructs

### 2. VirtualValue Abstraction (from cranelift-examples)

**Pattern**: Wrapper over Cranelift values to handle aggregates

- Scalars: Direct Cranelift values
- Structs: Either stack pointer or register collection
- Vectors: Treated as small structs (3-4 components)

**Benefit**: Uniform handling of different types, abstracts ABI complexity

### 3. Intrinsics Registry (from rustc_codegen_cranelift)

**Pattern**: HashMap of function name → implementations

- Multiple signatures per name (overloading)
- Different implementation strategies (libcall, inline, component-wise)

**Benefit**: Easy to add new built-ins, clear organization

### 4. Block-Based Control Flow (from lp-toy-lang)

**Pattern**: Create Cranelift blocks for each control flow construct

- If/else: then_block, else_block, merge_block
- Loops: header_block, body_block, exit_block
- Use block parameters for PHI nodes

**Benefit**: Maps cleanly to SSA form, leverages Cranelift's strengths

### 5. Two-Phase Type Handling (from DXC)

**Pattern**: Separate type checking from code generation

- Phase 1: Semantic analysis produces typed AST
- Phase 2: Codegen only deals with well-typed expressions

**Benefit**: Errors caught early, codegen can assume well-typed inputs

### 6. Post-Compilation Transformation (novel)

**Pattern**: Fixed-point transform runs on Cranelift IR after codegen

- Codegen generates standard float operations
- Transformation pass rewrites IR to use integer math
- Keeps codegen and fixed-point concerns separate

**Benefit**: Can toggle fixed-point transform on/off, easier to debug

## Module Dependency Graph

```
frontend (glsl-parser)
    ↓
semantic (type checking, symbol tables)
    ↓
ir (typed AST, lowering)
    ↓
codegen (Cranelift IR generation)
    ↓
[optional] fixed_point (IR transformation)
    ↓
Cranelift backend (RISC-V32 machine code)
```

## Compilation Flow Summary

1. **Parse**: GLSL text → glsl-parser AST
2. **Analyze**: AST → Typed AST (with semantic info)
3. **Lower**: Typed AST → Simplified IR (desugar complex constructs)
4. **Codegen**: Simplified IR → Cranelift IR (with float ops)
5. **Transform** (optional): Cranelift IR → Cranelift IR (with int ops)
6. **Compile**: Cranelift IR → RISC-V32 machine code
7. **Execute**: Machine code runs on target (emulator or hardware)

## Future Considerations

- Vertex shader support (different I/O model)
- Geometry shaders
- Compute shaders (more complex memory model)
- SPIR-V output (alternative backend)
- GPU codegen (non-RISC-V targets)
- Optimizations (constant folding, dead code elimination)
- Shader linking (multiple stages)
- WebGPU WGSL support (similar architecture)
