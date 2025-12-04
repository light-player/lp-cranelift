# GLSL Compiler Frontend - Phase 1

## Scope

Phase 1 establishes the **complete architecture** and implements **basic functionality**:

### What's Included

- ✅ Full module structure and architecture
- ✅ Type system (all types defined, but only int/bool implemented)
- ✅ Symbol table and scope management
- ✅ Semantic analysis infrastructure
- ✅ Cranelift codegen infrastructure
- ✅ Basic operations: arithmetic (+, -, \*, /), comparisons (==, !=, <, >)
- ✅ Variable declarations and assignments
- ✅ Integer and boolean types only
- ✅ Function with return value (int main() { ... return value; })
- ✅ JIT compilation and execution
- ✅ Filetest infrastructure (CLIF validation)
- ✅ Runtime tests (correctness validation)

### What's NOT Included

- ❌ Control flow (if/else, loops)
- ❌ Floating-point operations
- ❌ Vectors, matrices
- ❌ Structs
- ❌ Function calls (user-defined or built-ins)
- ❌ Arrays
- ❌ Fixed-point transformation

**Goal**: Validate the architecture works end-to-end with the simplest possible operations.

## Quick Example

Here's what Phase 1 can do:

```rust
use lp_glsl::Compiler;

fn main() {
    let mut compiler = Compiler::new();

    // Compile GLSL shader
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

This validates:

- ✅ GLSL parsing works
- ✅ Semantic analysis finds main() and validates types
- ✅ Cranelift IR generation works
- ✅ JIT compilation to native code works
- ✅ Execution returns correct result

---

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

---

## Crate Structure

Create `crates/lp-glsl/` with the following module organization:

- **`lib.rs`** - Main entry point, `#![no_std]` with std feature gate

  - Reference: `lp-toy-lang/src/lib.rs`

- **`frontend.rs`** - GLSL parsing (re-export from glsl-parser)

  - Uses: `/Users/yona/dev/photomancer/glsl-parser/glsl` crate
  - Parse to `glsl::syntax::TranslationUnit`

- **`semantic/mod.rs`** - Semantic analysis and type checking

  - `semantic/types.rs` - Type system (all types, Phase 1 uses int/bool only)
    - Reference: DXC `tools/clang/lib/AST/Type.cpp`, `HlslTypes.cpp`
  - `semantic/scope.rs` - Symbol table and scope management
    - Reference: DXC `tools/clang/lib/SPIRV/DeclResultIdMapper.h` (lines 1-150)
  - `semantic/validator.rs` - Semantic validation pass
  - `semantic/builtins.rs` - Stub for built-in registry (Phase 1: empty)

- **`codegen/mod.rs`** - Cranelift IR code generation

  - `codegen/context.rs` - Code generation context (variables, blocks)
    - Reference: rustc_codegen_cranelift `src/base.rs` FunctionCx pattern
  - `codegen/translator.rs` - Main AST-to-Cranelift translator
    - Reference: DXC `tools/clang/lib/SPIRV/SpirvEmitter.h` structure
  - `codegen/expr.rs` - Expression handlers (Phase 1: basic only)
  - `codegen/stmt.rs` - Statement handlers (Phase 1: declarations, assignments)

- **`jit.rs`** - JIT compilation interface

  - Reference: `lp-toy-lang/src/jit.rs` (lines 1-250)

- **`compiler.rs`** - Main compiler orchestration

---

## Compilation Pipeline

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

Phase 1 implements this full pipeline for the limited feature set, including execution and validation.

---

## Phase 1 Implementation Details

### Step 1: Project Setup

**Files to create:**

- `crates/lp-glsl/Cargo.toml`
- `crates/lp-glsl/src/lib.rs`

**Cargo.toml:**

```toml
[package]
name = "lp-glsl"
version.workspace = true
edition.workspace = true
license.workspace = true

[lints]
workspace = true

[dependencies]
glsl = { path = "/Users/yona/dev/photomancer/glsl-parser/glsl", default-features = false, features = ["alloc"] }
cranelift-frontend = { workspace = true }
cranelift-codegen = { workspace = true }
cranelift-jit = { workspace = true, optional = true }
cranelift-module = { workspace = true }
hashbrown = { workspace = true }

[features]
default = ["std"]
std = [
    "glsl/std",
    "cranelift-codegen/std",
    "cranelift-jit"
]
core = ["cranelift-codegen/core"]

[dev-dependencies]
cranelift-reader = { workspace = true }
```

**lib.rs:**

```rust
//! GLSL fragment shader compiler using Cranelift JIT.
//!
//! Phase 1: Basic architecture with int/bool support only.

#![no_std]

#[cfg(feature = "std")]
#[macro_use]
extern crate std;

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod frontend;
pub mod semantic;
pub mod codegen;
pub mod jit;
pub mod compiler;

pub use compiler::Compiler;
pub use jit::JIT;
```

### Step 2: Type System

**File: `crates/lp-glsl/src/semantic/types.rs`**

Reference: DXC `tools/clang/lib/AST/Type.cpp`

```rust
/// GLSL type system
/// Phase 1: Only Int and Bool are fully supported
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Void,
    Bool,
    Int,

    // Future phases:
    Float,
    Vec2, Vec3, Vec4,
    IVec2, IVec3, IVec4,
    BVec2, BVec3, BVec4,
    Mat2, Mat3, Mat4,
    Sampler2D,
    Struct(StructId),
    Array(Box<Type>, usize),
}

pub type StructId = usize;

impl Type {
    /// Returns true if this type is supported in Phase 1
    pub fn is_phase1_supported(&self) -> bool {
        matches!(self, Type::Void | Type::Bool | Type::Int)
    }

    /// Get the corresponding Cranelift type
    pub fn to_cranelift_type(&self) -> cranelift_codegen::ir::Type {
        match self {
            Type::Bool => cranelift_codegen::ir::types::I8,
            Type::Int => cranelift_codegen::ir::types::I32,
            _ => panic!("Type not supported in Phase 1"),
        }
    }
}
```

### Step 3: Symbol Table

**File: `crates/lp-glsl/src/semantic/scope.rs`**

Reference: DXC `DeclResultIdMapper.h`

```rust
use crate::semantic::types::Type;
use hashbrown::HashMap;

#[cfg(feature = "std")]
use std::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

pub struct SymbolTable {
    scopes: Vec<Scope>,
}

struct Scope {
    variables: HashMap<String, VarDecl>,
}

pub struct VarDecl {
    pub name: String,
    pub ty: Type,
    pub storage_class: StorageClass,
}

#[derive(Debug, Clone, Copy)]
pub enum StorageClass {
    Local,
    // Future: Uniform, In, Out
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope { variables: HashMap::new() }],
        }
    }

    pub fn declare_variable(&mut self, name: String, ty: Type, storage: StorageClass) -> Result<(), String> {
        let scope = self.scopes.last_mut().unwrap();
        if scope.variables.contains_key(&name) {
            return Err(format!("Variable '{}' already declared", name));
        }
        scope.variables.insert(name.clone(), VarDecl { name, ty, storage_class: storage });
        Ok(())
    }

    pub fn lookup_variable(&self, name: &str) -> Option<&VarDecl> {
        for scope in self.scopes.iter().rev() {
            if let Some(var) = scope.variables.get(name) {
                return Some(var);
            }
        }
        None
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope { variables: HashMap::new() });
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }
}
```

### Step 4: Codegen Context

**File: `crates/lp-glsl/src/codegen/context.rs`**

Reference: rustc_codegen_cranelift `src/base.rs` FunctionCx

```rust
use cranelift_codegen::ir::{types, Block, Value, Variable};
use cranelift_frontend::FunctionBuilder;
use cranelift_module::Module;
use hashbrown::HashMap;

pub struct CodegenContext<'a> {
    pub builder: FunctionBuilder<'a>,
    pub module: &'a mut dyn Module,
    pub variables: HashMap<String, Variable>,
    pub next_var: u32,
}

impl<'a> CodegenContext<'a> {
    pub fn new(builder: FunctionBuilder<'a>, module: &'a mut dyn Module) -> Self {
        Self {
            builder,
            module,
            variables: HashMap::new(),
            next_var: 0,
        }
    }

    pub fn declare_variable(&mut self, name: String, ty: types::Type) -> Variable {
        let var = Variable::new(self.next_var as usize);
        self.next_var += 1;
        self.builder.declare_var(var, ty);
        self.variables.insert(name, var);
        var
    }

    pub fn lookup_variable(&self, name: &str) -> Option<Variable> {
        self.variables.get(name).copied()
    }
}
```

### Step 5: Expression Translation (Phase 1 Subset)

**File: `crates/lp-glsl/src/codegen/expr.rs`**

Reference: DXC `SpirvEmitter.h` doExpr pattern

```rust
use cranelift_codegen::ir::{condcodes::IntCC, Value};
use glsl::syntax::Expr;
use crate::codegen::context::CodegenContext;

impl<'a> CodegenContext<'a> {
    pub fn translate_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            // Literals
            Expr::IntConst(n) => {
                let val = self.builder.ins().iconst(cranelift_codegen::ir::types::I32, *n as i64);
                Ok(val)
            }

            Expr::BoolConst(b) => {
                let val = self.builder.ins().iconst(cranelift_codegen::ir::types::I8, if *b { 1 } else { 0 });
                Ok(val)
            }

            // Variable reference
            Expr::Variable(ident) => {
                let var = self.lookup_variable(&ident.0)
                    .ok_or_else(|| format!("Variable '{}' not found", ident.0))?;
                let val = self.builder.use_var(var);
                Ok(val)
            }

            // Binary operators
            Expr::Binary(op, lhs, rhs) => {
                let lhs_val = self.translate_expr(lhs)?;
                let rhs_val = self.translate_expr(rhs)?;
                self.translate_binary_op(op, lhs_val, rhs_val)
            }

            // Unary operators
            Expr::Unary(op, expr) => {
                let val = self.translate_expr(expr)?;
                self.translate_unary_op(op, val)
            }

            // Assignment
            Expr::Assignment(lhs, op, rhs) => {
                self.translate_assignment(lhs, op, rhs)
            }

            _ => Err(format!("Expression not supported in Phase 1: {:?}", expr)),
        }
    }

    fn translate_binary_op(&mut self, op: &glsl::syntax::BinaryOp, lhs: Value, rhs: Value) -> Result<Value, String> {
        use glsl::syntax::BinaryOp::*;

        let val = match op {
            // Arithmetic
            Add => self.builder.ins().iadd(lhs, rhs),
            Sub => self.builder.ins().isub(lhs, rhs),
            Mult => self.builder.ins().imul(lhs, rhs),
            Div => self.builder.ins().sdiv(lhs, rhs),

            // Comparisons
            Equal => self.builder.ins().icmp(IntCC::Equal, lhs, rhs),
            NonEqual => self.builder.ins().icmp(IntCC::NotEqual, lhs, rhs),
            LT => self.builder.ins().icmp(IntCC::SignedLessThan, lhs, rhs),
            GT => self.builder.ins().icmp(IntCC::SignedGreaterThan, lhs, rhs),
            LTE => self.builder.ins().icmp(IntCC::SignedLessThanOrEqual, lhs, rhs),
            GTE => self.builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs),

            _ => return Err(format!("Binary operator not supported in Phase 1: {:?}", op)),
        };

        Ok(val)
    }

    fn translate_unary_op(&mut self, op: &glsl::syntax::UnaryOp, val: Value) -> Result<Value, String> {
        use glsl::syntax::UnaryOp::*;

        let result = match op {
            Minus => self.builder.ins().ineg(val),
            Not => {
                let zero = self.builder.ins().iconst(cranelift_codegen::ir::types::I8, 0);
                self.builder.ins().icmp(IntCC::Equal, val, zero)
            }
            _ => return Err(format!("Unary operator not supported in Phase 1: {:?}", op)),
        };

        Ok(result)
    }

    fn translate_assignment(&mut self, lhs: &Expr, op: &glsl::syntax::AssignmentOp, rhs: &Expr) -> Result<Value, String> {
        // Phase 1: Only simple assignment (=)
        if !matches!(op, glsl::syntax::AssignmentOp::Equal) {
            return Err("Only simple assignment (=) supported in Phase 1".to_string());
        }

        // Get variable name from lhs
        let var_name = match lhs {
            Expr::Variable(ident) => &ident.0,
            _ => return Err("Assignment lhs must be variable in Phase 1".to_string()),
        };

        let var = self.lookup_variable(var_name)
            .ok_or_else(|| format!("Variable '{}' not found", var_name))?;

        let rhs_val = self.translate_expr(rhs)?;
        self.builder.def_var(var, rhs_val);

        Ok(rhs_val)
    }
}
```

### Step 6: Statement Translation (Phase 1 Subset)

**File: `crates/lp-glsl/src/codegen/stmt.rs`**

```rust
use glsl::syntax::{Statement, SimpleStatement, Expr};
use crate::codegen::context::CodegenContext;
use crate::semantic::types::Type;

impl<'a> CodegenContext<'a> {
    pub fn translate_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::Simple(simple) => self.translate_simple_statement(simple),
            _ => Err("Only simple statements supported in Phase 1".to_string()),
        }
    }

    fn translate_simple_statement(&mut self, stmt: &SimpleStatement) -> Result<(), String> {
        match stmt {
            SimpleStatement::Declaration(decl) => {
                self.translate_declaration(decl)
            }
            SimpleStatement::Expression(Some(expr)) => {
                self.translate_expr(expr)?;
                Ok(())
            }
            SimpleStatement::Expression(None) => Ok(()), // Empty statement
            _ => Err("Statement type not supported in Phase 1".to_string()),
        }
    }

    fn translate_declaration(&mut self, decl: &glsl::syntax::Declaration) -> Result<(), String> {
        use glsl::syntax::Declaration;

        match decl {
            Declaration::InitDeclaratorList(list) => {
                // Phase 1: Only handle simple int/bool declarations
                for declarator in &list.head.0 {
                    // TODO: Parse type from list.head type specifier
                    // For now, assume int
                    let ty = cranelift_codegen::ir::types::I32;
                    let var = self.declare_variable(declarator.name.0.clone(), ty);

                    // Handle initializer if present
                    if let Some(init) = &declarator.initializer {
                        let init_val = self.translate_initializer(init)?;
                        self.builder.def_var(var, init_val);
                    }
                }
                Ok(())
            }
            _ => Err("Only variable declarations supported in Phase 1".to_string()),
        }
    }

    fn translate_initializer(&mut self, init: &glsl::syntax::Initializer) -> Result<cranelift_codegen::ir::Value, String> {
        use glsl::syntax::Initializer;

        match init {
            Initializer::Simple(expr) => self.translate_expr(expr),
            _ => Err("Only simple initializers supported in Phase 1".to_string()),
        }
    }
}
```

### Step 7: JIT Interface and Compiler

**File: `crates/lp-glsl/src/jit.rs`**

Reference: `lp-toy-lang/src/jit.rs`

```rust
use cranelift_codegen::ir::{AbiParam, Signature};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::Context as CodegenContext;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataDescription, Linkage, Module};

pub struct JIT {
    builder_context: FunctionBuilderContext,
    ctx: CodegenContext,
    data_description: DataDescription,
    module: JITModule,
}

impl Default for JIT {
    fn default() -> Self {
        Self::new()
    }
}

impl JIT {
    #[cfg(feature = "std")]
    pub fn new() -> Self {
        use cranelift_codegen::settings;

        let mut flag_builder = settings::builder();
        flag_builder.set("is_pic", "false").expect("set is_pic");
        flag_builder.set("use_colocated_libcalls", "false").expect("set use_colocated_libcalls");

        let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
            panic!("host machine is not supported: {}", msg);
        });

        let isa = isa_builder.finish(settings::Flags::new(flag_builder)).unwrap();
        let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        let module = JITModule::new(builder);

        Self {
            builder_context: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            data_description: DataDescription::new(),
            module,
        }
    }

    /// Compile GLSL source to machine code and return function pointer
    pub fn compile(&mut self, glsl_source: &str) -> Result<*const u8, String> {
        // 1. Parse GLSL
        let shader = glsl::parser::Parse::parse(glsl_source)
            .map_err(|e| format!("Parse error: {:?}", e))?;

        // 2. Semantic analysis
        let typed_ast = crate::semantic::analyze(&shader)?;

        // 3. Generate Cranelift IR
        self.translate(typed_ast)?;

        // 4. Declare function
        let id = self.module
            .declare_function("main", Linkage::Export, &self.ctx.func.signature)
            .map_err(|e| e.to_string())?;

        // 5. Define function
        self.module
            .define_function(id, &mut self.ctx)
            .map_err(|e| e.to_string())?;

        // 6. Finalize
        self.module.clear_context(&mut self.ctx);
        self.module.finalize_definitions();

        // 7. Get function pointer
        let code = self.module.get_finalized_function(id);
        Ok(code)
    }

    /// Compile and return CLIF IR as string (for filetests)
    pub fn compile_to_clif(&mut self, glsl_source: &str) -> Result<String, String> {
        self.ctx.clear();

        // 1. Parse GLSL
        let shader = glsl::parser::Parse::parse(glsl_source)
            .map_err(|e| format!("Parse error: {:?}", e))?;

        // 2. Semantic analysis
        let typed_ast = crate::semantic::analyze(&shader)?;

        // 3. Generate Cranelift IR
        self.translate(typed_ast)?;

        // 4. Return as string
        Ok(format!("{}", self.ctx.func))
    }

    fn translate(&mut self, typed_ast: crate::semantic::TypedShader) -> Result<(), String> {
        // Create function signature: () -> i32
        let int_type = self.module.target_config().pointer_type();
        self.ctx.func.signature.returns.push(AbiParam::new(int_type));

        // Create function builder
        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);

        // Create entry block
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        // Create codegen context
        let mut ctx = crate::codegen::context::CodegenContext::new(builder, &mut self.module);

        // Translate main function body
        for stmt in typed_ast.main_function.body {
            ctx.translate_statement(&stmt)?;
        }

        // Get return value (if any)
        let return_val = if let Some(ret_expr) = typed_ast.main_function.return_expr {
            ctx.translate_expr(&ret_expr)?
        } else {
            // Default return 0
            ctx.builder.ins().iconst(int_type, 0)
        };

        // Emit return
        ctx.builder.ins().return_(&[return_val]);

        // Finalize
        ctx.builder.finalize();
        Ok(())
    }
}

/// Execute a compiled function that returns i32
pub fn execute_i32(code_ptr: *const u8) -> i32 {
    let func: fn() -> i32 = unsafe { std::mem::transmute(code_ptr) };
    func()
}

/// Execute a compiled function that returns bool (as i8)
pub fn execute_bool(code_ptr: *const u8) -> bool {
    let func: fn() -> i8 = unsafe { std::mem::transmute(code_ptr) };
    func() != 0
}
```

**File: `crates/lp-glsl/src/compiler.rs`**

```rust
/// High-level compiler interface
pub struct Compiler {
    jit: crate::jit::JIT,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            jit: crate::jit::JIT::new(),
        }
    }

    /// Compile GLSL shader that returns i32
    pub fn compile_int(&mut self, source: &str) -> Result<fn() -> i32, String> {
        let code_ptr = self.jit.compile(source)?;
        Ok(unsafe { std::mem::transmute(code_ptr) })
    }

    /// Compile GLSL shader that returns bool
    pub fn compile_bool(&mut self, source: &str) -> Result<fn() -> bool, String> {
        let code_ptr = self.jit.compile(source)?;
        let func: fn() -> i8 = unsafe { std::mem::transmute(code_ptr) };
        Ok(move || func() != 0)
    }
}
```

### Step 8: Semantic Analysis Module

**File: `crates/lp-glsl/src/semantic/mod.rs`**

```rust
use glsl::syntax::TranslationUnit;

pub mod types;
pub mod scope;
pub mod validator;

pub struct TypedShader {
    pub main_function: TypedFunction,
}

pub struct TypedFunction {
    pub body: Vec<glsl::syntax::Statement>,
    pub return_expr: Option<glsl::syntax::Expr>,
}

/// Analyze GLSL shader and produce typed AST
pub fn analyze(shader: &TranslationUnit) -> Result<TypedShader, String> {
    // Phase 1: Just extract main() function and basic validation
    // TODO: Full semantic analysis in later implementation

    // Find main function
    for decl in &shader.0 {
        if let glsl::syntax::ExternalDeclaration::FunctionDefinition(func) = decl {
            if let glsl::syntax::FunctionPrototype { name, .. } = &func.prototype.prototype {
                if name.0 == "main" {
                    // Extract body statements and return expression
                    let (body, return_expr) = extract_function_body(&func.statement);

                    return Ok(TypedShader {
                        main_function: TypedFunction {
                            body,
                            return_expr,
                        },
                    });
                }
            }
        }
    }

    Err("No main() function found".to_string())
}

fn extract_function_body(stmt: &glsl::syntax::Statement) -> (Vec<glsl::syntax::Statement>, Option<glsl::syntax::Expr>) {
    use glsl::syntax::Statement;

    match stmt {
        Statement::Compound(compound) => {
            let mut body = Vec::new();
            let mut return_expr = None;

            for stmt in &compound.statement_list {
                // Check if this is a return statement
                if let Statement::Simple(simple) = stmt {
                    if let glsl::syntax::SimpleStatement::Jump(jump) = simple.as_ref() {
                        if let glsl::syntax::JumpStatement::Return(expr) = jump {
                            return_expr = expr.clone();
                            continue;
                        }
                    }
                }
                body.push(stmt.clone());
            }

            (body, return_expr)
        }
        _ => (vec![], None),
    }
}
```

---

## Testing Infrastructure

Phase 1 includes two types of tests:

1. **Filetests** - Validate CLIF IR output (compile-time correctness)
2. **Runtime Tests** - Execute code and validate results (runtime correctness)

---

## Filetest Infrastructure

**Reference**: Cranelift filetests in `cranelift/filetests/`

### Directory Structure

```
crates/lp-glsl/filetests/
├── basic/
│   ├── int_literal.glsl
│   ├── bool_literal.glsl
│   ├── arithmetic.glsl
│   ├── comparisons.glsl
│   ├── variables.glsl
│   └── assignment.glsl
└── runone.rs  (test runner)
```

### Example Filetest: `filetests/basic/arithmetic.glsl`

```glsl
; Test: Basic integer arithmetic
; CHECK: function

void main() {
    int a = 10;
    int b = 20;
    int sum = a + b;
    int diff = a - b;
    int prod = a * b;
    int quot = b / a;
}

; CHECK: iconst
; CHECK: iadd
; CHECK: isub
; CHECK: imul
; CHECK: sdiv
```

### Test Runner: `filetests/runone.rs`

```rust
use std::path::Path;
use cranelift_reader::{parse_test, TestCommand};

pub fn run_filetest(path: &Path) -> Result<(), String> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    // Parse GLSL, compile to CLIF
    let mut jit = lp_glsl::JIT::new();
    let clif = jit.compile_to_clif(&contents)?;

    // Check expectations
    let test = parse_test(&contents)
        .map_err(|e| format!("Failed to parse test: {}", e))?;

    for command in test.commands {
        match command {
            TestCommand::Check(pattern) => {
                if !clif.contains(&pattern) {
                    return Err(format!("Pattern '{}' not found in CLIF output", pattern));
                }
            }
            _ => {}
        }
    }

    Ok(())
}
```

---

## Runtime Test Infrastructure

**Reference**: Integration testing pattern from lp-toy-lang

### Directory Structure

```
crates/lp-glsl/tests/
├── runtime_int.rs      (integer return value tests)
├── runtime_bool.rs     (boolean return value tests)
└── common/
    └── mod.rs          (shared test utilities)
```

### Test Utilities: `tests/common/mod.rs`

```rust
use lp_glsl::Compiler;

/// Compile and execute GLSL that returns i32
pub fn run_int_test(source: &str) -> i32 {
    let mut compiler = Compiler::new();
    let func = compiler.compile_int(source)
        .expect("Compilation failed");
    func()
}

/// Compile and execute GLSL that returns bool
pub fn run_bool_test(source: &str) -> bool {
    let mut compiler = Compiler::new();
    let func = compiler.compile_bool(source)
        .expect("Compilation failed");
    func()
}

/// Assert that GLSL code produces expected integer result
#[macro_export]
macro_rules! assert_int_result {
    ($source:expr, $expected:expr) => {
        let result = common::run_int_test($source);
        assert_eq!(result, $expected,
            "Expected {}, got {} for:\n{}", $expected, result, $source);
    };
}

/// Assert that GLSL code produces expected boolean result
#[macro_export]
macro_rules! assert_bool_result {
    ($source:expr, $expected:expr) => {
        let result = common::run_bool_test($source);
        assert_eq!(result, $expected,
            "Expected {}, got {} for:\n{}", $expected, result, $source);
    };
}
```

### Runtime Tests: `tests/runtime_int.rs`

```rust
mod common;

#[test]
fn test_int_literal() {
    assert_int_result!(r#"
        int main() {
            return 42;
        }
    "#, 42);
}

#[test]
fn test_int_addition() {
    assert_int_result!(r#"
        int main() {
            int a = 10;
            int b = 20;
            return a + b;
        }
    "#, 30);
}

#[test]
fn test_int_subtraction() {
    assert_int_result!(r#"
        int main() {
            int a = 50;
            int b = 20;
            return a - b;
        }
    "#, 30);
}

#[test]
fn test_int_multiplication() {
    assert_int_result!(r#"
        int main() {
            int a = 6;
            int b = 7;
            return a * b;
        }
    "#, 42);
}

#[test]
fn test_int_division() {
    assert_int_result!(r#"
        int main() {
            int a = 84;
            int b = 2;
            return a / b;
        }
    "#, 42);
}

#[test]
fn test_int_complex_expression() {
    assert_int_result!(r#"
        int main() {
            int a = 5;
            int b = 3;
            int c = 2;
            return (a + b) * c - 4;
        }
    "#, 12);  // (5 + 3) * 2 - 4 = 16 - 4 = 12
}

#[test]
fn test_int_negative() {
    assert_int_result!(r#"
        int main() {
            int a = 10;
            return -a;
        }
    "#, -10);
}

#[test]
fn test_int_assignment_chain() {
    assert_int_result!(r#"
        int main() {
            int a = 5;
            int b = a + 10;
            int c = b * 2;
            return c;
        }
    "#, 30);  // (5 + 10) * 2 = 30
}
```

### Runtime Tests: `tests/runtime_bool.rs`

```rust
mod common;

#[test]
fn test_bool_true() {
    assert_bool_result!(r#"
        bool main() {
            return true;
        }
    "#, true);
}

#[test]
fn test_bool_false() {
    assert_bool_result!(r#"
        bool main() {
            return false;
        }
    "#, false);
}

#[test]
fn test_bool_not() {
    assert_bool_result!(r#"
        bool main() {
            bool t = true;
            return !t;
        }
    "#, false);
}

#[test]
fn test_int_comparison_eq() {
    assert_bool_result!(r#"
        bool main() {
            int a = 42;
            int b = 42;
            return a == b;
        }
    "#, true);
}

#[test]
fn test_int_comparison_ne() {
    assert_bool_result!(r#"
        bool main() {
            int a = 10;
            int b = 20;
            return a != b;
        }
    "#, true);
}

#[test]
fn test_int_comparison_lt() {
    assert_bool_result!(r#"
        bool main() {
            int a = 10;
            int b = 20;
            return a < b;
        }
    "#, true);
}

#[test]
fn test_int_comparison_gt() {
    assert_bool_result!(r#"
        bool main() {
            int a = 30;
            int b = 20;
            return a > b;
        }
    "#, true);
}

#[test]
fn test_int_comparison_complex() {
    assert_bool_result!(r#"
        bool main() {
            int a = 5;
            int b = 10;
            int c = 15;
            return (a + b) == c;
        }
    "#, true);
}
```

---

## Phase 1 Test Cases

### Test 1: Integer Return Value

**GLSL:**

```glsl
int main() {
    return 42;
}
```

**Expected CLIF:**

```clif
function u0:0() -> i32 {
block0:
    v0 = iconst.i32 42
    return v0
}
```

**Expected Runtime Result:** `42`

---

### Test 2: Integer Arithmetic

**GLSL:**

```glsl
int main() {
    int a = 10;
    int b = 5;
    int sum = a + b;
    return sum;
}
```

**Expected CLIF Patterns:**

- `iconst.i32 10`
- `iconst.i32 5`
- `iadd`

**Expected Runtime Result:** `15`

---

### Test 3: Integer Expression

**GLSL:**

```glsl
int main() {
    int a = 6;
    int b = 7;
    return a * b;
}
```

**Expected CLIF Patterns:**

- `imul`

**Expected Runtime Result:** `42`

---

### Test 4: Boolean Comparison

**GLSL:**

```glsl
bool main() {
    int a = 10;
    int b = 20;
    return a < b;
}
```

**Expected CLIF Patterns:**

- `icmp slt`

**Expected Runtime Result:** `true`

---

### Test 5: Complex Expression

**GLSL:**

```glsl
int main() {
    int a = 5;
    int b = 3;
    int c = 2;
    return (a + b) * c - 4;
}
```

**Expected CLIF Patterns:**

- `iadd`
- `imul`
- `isub`

**Expected Runtime Result:** `12` (i.e., (5+3)\*2-4 = 16-4 = 12)

---

### Test 6: Unary Operators

**GLSL:**

```glsl
int main() {
    int x = 42;
    int neg = -x;
    return neg;
}
```

**Expected CLIF Patterns:**

- `ineg`

**Expected Runtime Result:** `-42`

---

### Test 7: Boolean NOT

**GLSL:**

```glsl
bool main() {
    bool t = true;
    return !t;
}
```

**Expected CLIF Patterns:**

- `icmp eq`

**Expected Runtime Result:** `false`

---

### Test 8: Comparison Chain

**GLSL:**

```glsl
bool main() {
    int a = 5;
    int b = 10;
    int c = 15;
    return (a + b) == c;
}
```

**Expected Runtime Result:** `true`

---

## Success Criteria for Phase 1

### Compilation

- [ ] Can parse basic GLSL with int/bool types
- [ ] Symbol table tracks variables correctly
- [ ] Semantic analysis validates types and finds main()
- [ ] Codegen produces valid Cranelift IR
- [ ] Can output CLIF IR for inspection
- [ ] Can compile to machine code

### Testing Infrastructure

- [ ] Filetest infrastructure works (CLIF validation)
- [ ] Runtime test infrastructure works (JIT execution)
- [ ] Test utilities compile and run
- [ ] Can execute compiled code and get results

### Correctness

- [ ] All 8 compile test cases produce correct CLIF
- [ ] All 8 runtime test cases produce correct results
- [ ] Integer arithmetic works correctly (+, -, \*, /, neg)
- [ ] Integer comparisons work correctly (==, !=, <, >, <=, >=)
- [ ] Boolean operations work correctly (literals, !)
- [ ] Variable declarations and assignments work
- [ ] Return statements work for int and bool

### Architecture

- [ ] Module structure is clean and follows plan
- [ ] Code follows Rust best practices
- [ ] Architecture is extensible for future phases
- [ ] Documentation is clear and helpful

---

## Implementation Order

1. Project setup

   - Create `Cargo.toml` with dependencies
   - Set up `lib.rs` module structure
   - Verify glsl-parser integration

2. Type system and symbol table

   - Implement `semantic/types.rs`
   - Implement `semantic/scope.rs`
   - Add basic validation

3. Semantic analysis

   - Implement `semantic/mod.rs` with shader analyzer
   - Extract main() function
   - Basic AST traversal

4. Codegen infrastructure

   - Implement `codegen/context.rs`
   - Set up FunctionBuilder pattern
   - Create variable mapping

5. Expression translation - Part 1

   - Implement `codegen/expr.rs`
   - Literals (int, bool)
   - Variable references
   - Test with simple expressions

6. Expression translation - Part 2

   - Binary operators (arithmetic)
   - Binary operators (comparisons)
   - Unary operators
   - Assignment

7. Statement translation

   - Implement `codegen/stmt.rs`
   - Variable declarations
   - Expression statements
   - Return statements

8. JIT compilation

   - Implement `jit.rs` with full pipeline
   - Implement `compiler.rs` high-level API
   - Test CLIF output
   - Test JIT execution

9. Test infrastructure

   - Set up filetests with CHECK patterns
   - Set up runtime tests (int and bool)
   - Implement test utilities
   - Create test cases

10. Testing and validation
    - Run all filetests
    - Run all runtime tests
    - Fix bugs
    - Document architecture
    - Validate all success criteria

---

## Key Architectural Patterns (Phase 1)

All patterns from the full plan apply, but we only implement the core dispatcher pattern:

### Dispatcher Pattern

```rust
match expr {
    Expr::IntConst(n) => /* ... */,
    Expr::BoolConst(b) => /* ... */,
    Expr::Variable(v) => /* ... */,
    Expr::Binary(op, lhs, rhs) => /* ... */,
    Expr::Unary(op, expr) => /* ... */,
    Expr::Assignment(lhs, op, rhs) => /* ... */,
    _ => Err("Not supported in Phase 1"),
}
```

This validates the architecture without implementing all the complexity.
