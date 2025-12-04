# ISLE RV64→RV32 Migration Progress

## Completed ✓

### Step 1: Backup ✓
- Created backup at `.backups/riscv32-isle-20251203-153005/`

### Steps 2-3: inst.isle Transformations ✓ (Committed: 7dafceaeb)
- ✓ Removed RV64-specific enum variants:
  - AluOPRRR: Addw, Subw, Sllw, Srlw, Sraw, Mulw, Divw, Divuw, Remw, Remuw, Rolw, Rorw, Packw
  - AluOPRRI: Addiw, Slliw, SrliW, Sraiw, Clzw, Ctzw, Cpopw, Roriw
  - CaOp: CAddw, CSubw
  - CiOp: CAddiw
  - Kept AtomicOP *W variants (correct for RV32A)

- ✓ Removed RV64-specific helper functions:
  - rv_addw, rv_addiw, rv_sextw, rv_subw
  - rv_sllw, rv_slliw, rv_srlw, rv_srliw, rv_sraw, rv_sraiw
  - rv_mulw, rv_divw, rv_divuw, rv_remw, rv_remuw
  - rv_clzw, rv_ctzw, rv_cpopw, rv_rolw, rv_rorw, rv_roriw, rv_packw

- ✓ Fixed type selection and utility functions:
  - select_addi: Always use Addi on RV32
  - lower_ctz: Use rv_ctz for all types (not rv_ctzw)
  - zext: Removed packw/rv_zextw rules, simplified for RV32
  - sext: Removed rv_sextw rule for I32 (already native width)

- ✓ Vector support stubs:
  - Added float_int_max to inst_vector.isle
  - Commented out vector bitcast rules (deferred to Phase 2)

## In Progress 🚧

### Steps 4-6: lower.isle Transformations (Partially Done)

The following ALU operation updates are needed in lower.isle:

1. **iadd**: Change from rv_addw to rv_add for fits_in_32 types
2. **isub**: Change from rv_subw to rv_sub for fits_in_32 types  
3. **imul**: Change from rv_mulw to rv_mul for fits_in_32 types
4. **udiv**: Change from rv_divuw to rv_divu for all types
5. **sdiv**: Change from rv_divw to rv_div for all types
6. **urem**: Change from rv_remuw to rv_remu for all types
7. **srem**: Change from rv_remw to rv_rem for all types
8. **ishl**: Change from rv_sllw/rv_slliw to rv_sll/rv_slli
9. **ushr**: Change from rv_srlw/rv_srliw to rv_srl/rv_srli
10. **sshr**: Change from rv_sraw/rv_sraiw to rv_sra/rv_srai

### Vector Rules to Comment Out

All rules matching `(rule.*ty_supported_vec` need to be fully commented out:
- ~422 lines across ~175 rules
- Affects: iadd, isub, imul, shifts, and many other vector operations
- Can be done with automated script

## Helper Script for lower.isle

```python
#!/usr/bin/env python3
# Comment out all vector rules in lower.isle

import re

with open('cranelift/codegen/src/isa/riscv32/lower.isle', 'r') as f:
    lines = f.readlines()

output = []
in_vector_rule = False
paren_depth = 0

for line in lines:
    # Detect start of vector rule
    if re.match(r'^\(rule.*ty_supported_vec', line) and not line.startswith(';;'):
        in_vector_rule = True
        paren_depth = 0
    
    # Track parentheses depth if in vector rule
    if in_vector_rule:
        paren_depth += line.count('(') - line.count(')')
        output.append(';; ' + line)
        # End of rule when parens balance
        if paren_depth == 0:
            in_vector_rule = False
    else:
        output.append(line)

with open('cranelift/codegen/src/isa/riscv32/lower.isle', 'w') as f:
    f.writelines(output)

print("Vector rules commented out")
```

## Next Steps

1. Run the helper script above to comment out all vector rules
2. Apply the ALU operation transformations listed above
3. Test compilation: `cargo build --package cranelift-codegen --features riscv32,std`
4. Verify generated code uses correct opcodes (0x33 for OP, not 0x3b for OP-32)
5. Run cranelift filetests for riscv32

## Success Criteria

✓ All three ISLE files compile successfully  
✓ No RV64-specific `*w` instruction variants in enums (except atomics)  
✓ No `rv_*w` helper functions (except atomic-related)  
✓ Type selection functions don't use `fits_in_32` to choose RV64 instructions  
☐ Generated code uses opcode 0x33 (OP) for integer ops, not 0x3b (OP-32)  
☐ Simple iadd/isub/imul operations generate correct RV32 instructions

