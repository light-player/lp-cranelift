# GLSL Compiler - Remaining Work After Phase 1

## Overview

Phase 1 establishes the architecture with int/bool support. The remaining work adds:

1. **Floating-point operations** (Phase 2)
2. **Control flow** (Phase 3)
3. **Vectors and built-ins** (Phase 4)
4. **Matrices** (Phase 5)
5. **User functions and structs** (Phase 6)
6. **Fixed-point transformation** (Phase 7)
7. **RISC-V integration** (Phase 8)

---

## Phase 2: Floating-Point Operations

### Scope
- Add `float` type support
- Float literals and constants
- Float arithmetic (+, -, *, /)
- Float comparisons
- Type conversions (int ↔ float)

### Reference
- DXC float handling
- Cranelift F32 operations
- lp-toy-lang patterns apply directly

### Estimated Effort
2-3 days

---

## Phase 3: Control Flow

### Scope
- If/else statements
- While loops
- For loops
- Do-while loops
- Break and continue
- Early return

### Reference
- DXC `SpirvEmitter.h` lines 139-149 (statement handlers)
- lp-toy-lang if/else pattern (lines 333-392)
- lp-toy-lang while loop pattern (lines 394-424)

### Estimated Effort
5-7 days

---

## Phase 4: Vectors and Built-ins

### Scope
- vec2, vec3, vec4 types
- Vector construction
- Swizzling (read and write)
- Component-wise operations
- Vector built-ins: dot, cross, length, normalize, reflect
- Math built-ins: sin, cos, sqrt, pow, etc.

### Reference
- cranelift-examples `lowering-structs/` for vector representation
- rustc_codegen_cranelift SIMD patterns
- DXC `doHLSLVectorElementExpr` for swizzling

### Estimated Effort
10-14 days

---

## Phase 5: Matrices

### Scope
- mat2, mat3, mat4 types
- Matrix construction
- Matrix-vector multiplication
- Matrix-matrix multiplication
- Matrix built-ins: transpose, matrixCompMult

### Reference
- Column-major storage (GLSL convention)
- Expand to vector operations

### Estimated Effort
5-7 days

---

## Phase 6: User Functions and Structs

### Scope
- User-defined functions
- Function parameters and returns
- Inout/out parameters
- User-defined structs
- Struct member access
- Nested structs

### Reference
- cranelift-examples struct lowering (complete reference)
- DXC function call handling
- ABI considerations from rustc_codegen_cranelift

### Estimated Effort
10-14 days

---

## Phase 7: Fixed-Point Transformation

### Scope
- Cranelift IR analysis pass
- F32 → I32 type replacement
- Operator replacement (fadd → fixed_add, etc.)
- Fixed-point runtime library
- Math function implementations (sin, cos, sqrt)

### Reference
- RISC-V soft-float emulation
- CORDIC algorithms for trigonometry
- 16.16 fixed-point format

### Estimated Effort
14-21 days

---

## Phase 8: RISC-V Integration

### Scope
- Configure Cranelift for RISC-V32 IMAC
- Wire up to lp-riscv-tools emulator
- End-to-end testing
- Performance validation
- Precision validation for fixed-point

### Reference
- lp-riscv-tools emulator
- RISC-V ABI specifications

### Estimated Effort
7-10 days

---

## Future Work (Beyond Phase 8)

- Vertex shader support
- Texture sampling (runtime integration)
- Shader I/O (uniforms, varyings)
- Geometry shaders
- Compute shaders
- SPIR-V output backend
- Optimizations (constant folding, dead code elimination)
- WebGPU WGSL support

---

## Total Estimated Timeline

- **Phase 1**: 10 days (architecture + basic ops)
- **Phase 2**: 3 days (floats)
- **Phase 3**: 7 days (control flow)
- **Phase 4**: 14 days (vectors + built-ins)
- **Phase 5**: 7 days (matrices)
- **Phase 6**: 14 days (functions + structs)
- **Phase 7**: 21 days (fixed-point)
- **Phase 8**: 10 days (RISC-V integration)

**Total**: ~86 days (~3-4 months of focused development)

---

## Key Dependencies

Each phase builds on the previous:

```
Phase 1 (architecture)
    ↓
Phase 2 (floats) ← required for Phase 4
    ↓
Phase 3 (control flow) ← makes shaders useful
    ↓
Phase 4 (vectors) ← core GLSL feature
    ↓
Phase 5 (matrices) ← depends on vectors
    ↓
Phase 6 (functions + structs) ← depends on types
    ↓
Phase 7 (fixed-point) ← transforms existing codegen
    ↓
Phase 8 (RISC-V) ← validates everything
```

---

## Validation Strategy

After each phase:
1. Add filetests for new features
2. Add integration tests
3. Validate CLIF output
4. Test on real GLSL shader snippets
5. Update documentation

Final validation (Phase 8):
- Run complete shaders on RISC-V emulator
- Compare fixed-point vs float precision
- Benchmark performance
- Test edge cases

