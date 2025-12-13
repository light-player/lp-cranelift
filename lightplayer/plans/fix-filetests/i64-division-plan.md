# i64 Division/Remainder Implementation Plan for RISC-V32

## Goal
Implement full 64-bit division and remainder operations for RISC-V32 using register pairs.

## Algorithm Overview

### Unsigned Division (udiv)
For dividing 64-bit dividend by 64-bit divisor:

**Algorithm**: Binary long division (restoring division)
1. Initialize: quotient = 0, remainder = dividend
2. For each bit position from 63 down to 0:
   - Shift remainder left by 1 (bring in next bit from dividend)
   - If remainder >= divisor:
     - remainder = remainder - divisor
     - Set corresponding bit in quotient

**Optimization**: If divisor fits in 32 bits (high 32 bits are 0):
- Use simpler two-step algorithm:
  1. Divide high 32 bits by divisor → quotient_high, remainder_high
  2. Combine (remainder_high << 32) | low_32, divide by divisor → quotient_low, remainder
  3. Result: (quotient_high, quotient_low)

### Signed Division (sdiv)
1. Convert to unsigned: take absolute values, remember signs
2. Perform unsigned division
3. Apply sign to result

### Remainder (urem/srem)
- Remainder is the final remainder from division algorithm
- For signed remainder, apply sign of dividend

## Implementation Strategy

### Step 1: Create Helper Functions in inst.isle

```isle
;; 64-bit unsigned division helper
(decl div_i64_unsigned (ValueRegs ValueRegs) ValueRegs)
;; Returns (quotient_low, quotient_high)

;; 64-bit signed division helper  
(decl div_i64_signed (ValueRegs ValueRegs) ValueRegs)

;; 64-bit unsigned remainder helper
(decl rem_i64_unsigned (ValueRegs ValueRegs) ValueRegs)
;; Returns (remainder_low, remainder_high)

;; 64-bit signed remainder helper
(decl rem_i64_signed (ValueRegs ValueRegs) ValueRegs)
```

### Step 2: Implement Binary Long Division

The core algorithm needs:
- Shift left operation for 64-bit values (register pairs)
- Compare operation for 64-bit values (already exists)
- Subtract operation for 64-bit values (already exists via sub_i64)
- Bit manipulation for setting quotient bits

**Key Challenge**: ISLE doesn't easily support loops, so we need to unroll or use a different approach.

**Alternative Approach**: Use a helper function that emits a sequence of instructions. This might require creating a new MInst variant or using a library call.

### Step 3: Optimize for Common Cases

1. **Divisor fits in 32 bits**: Use simpler two-step algorithm
2. **Divisor is power of 2**: Use shift instead
3. **Small constant divisors**: Use multiplication-based approach

## Reference Implementation

Looking at how other architectures handle this:
- x64: Native double-width division (not applicable)
- ARM: Software implementation using multiple instructions
- Standard approach: Binary long division with 64 iterations (one per bit)

## Implementation Notes

1. **ISLE Limitations**: ISLE doesn't support loops, so we need to either:
   - Unroll all 64 iterations (very verbose)
   - Create a helper MInst that handles the loop
   - Use a library call (runtime function)

2. **Recommended Approach**: Create a helper MInst `DivI64` that:
   - Takes dividend (ValueRegs) and divisor (ValueRegs)
   - Returns quotient (ValueRegs) and remainder (ValueRegs)
   - Emits a sequence of instructions in the backend

3. **For Now**: Implement the optimized case (divisor fits in 32 bits) which covers most practical cases, then add full support later.

## Testing

Test cases from urem.clif:
- `%urem_i64(0, 1) == 0`
- `%urem_i64(2, 2) == 0`
- `%urem_i64(1, -1) == 1`  (signed)
- `%urem_i64(3, 2) == 1`
- `%urem_i64(19, 7) == 5`
- `%urem_i64(0xC0FFEEEE_DECAFFFF, 8) == 7`  (64-bit divisor)

