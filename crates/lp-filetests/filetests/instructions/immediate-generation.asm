test instruction

# ============================================================================
# LUI Tests - Load Upper Immediate
# ============================================================================

# Test LUI: basic positive value
# lui a0, 0x12345 should load 0x12345000 = 305418240
lui a0, 0x12345
ebreak

; check: Registers after execution:
; check: a0 = 305418240

# Test LUI: zero immediate
lui a0, 0
ebreak

; check: Registers after execution:
; check: a0 = 0

# Test LUI: small positive value (0x1)
# Should load 0x1000 = 4096
lui a0, 0x1
ebreak

; check: Registers after execution:
; check: a0 = 4096

# Test LUI: larger positive value (0x7FFFF - max positive 20-bit)
# Should load 0x7FFFF000 = 2147479552
lui a0, 0x7FFFF
ebreak

; check: Registers after execution:
; check: a0 = 2147479552

# Test LUI: value with sign bit set (0x80000 - min negative 20-bit)
# Should load 0x80000000 = -2147483648 (minimum i32)
lui a0, 0x80000
ebreak

; check: Registers after execution:
; check: a0 = -2147483648

# Test LUI: maximum 20-bit value (0xFFFFF - all bits set)
# Should load 0xFFFFF000 = -4096
lui a0, 0xFFFFF
ebreak

; check: Registers after execution:
; check: a0 = -4096

# Test LUI: negative value (0xFF000)
# Should load 0xFF000000 = -16777216
lui a0, 0xFF000
ebreak

; check: Registers after execution:
; check: a0 = -16777216

# Test LUI: different register
lui a1, 0x12345
ebreak

; check: Registers after execution:
; check: a1 = 305418240

# Test LUI: multiple LUI instructions
lui a0, 0x10000
lui a1, 0x20000
lui a2, 0x30000
ebreak

; check: Registers after execution:
; check: a0 = 268435456
; check: a1 = 536870912
; check: a2 = 805306368

# ============================================================================
# AUIPC Tests - Add Upper Immediate to PC
# ============================================================================

# Test AUIPC: zero immediate at PC=0
# auipc a0, 0 should load PC + 0 = 0
.org 0x0
auipc a0, 0
ebreak

; check: Registers after execution:
; check: a0 = 0

# Test AUIPC: positive immediate at PC=0
# auipc a0, 0x1 should load PC + (0x1 << 12) = 0 + 4096 = 4096
.org 0x0
auipc a0, 0x1
ebreak

; check: Registers after execution:
; check: a0 = 4096

# Test AUIPC: larger positive immediate at PC=0
# auipc a0, 0x100 should load PC + (0x100 << 12) = 0 + 0x100000 = 1048576
.org 0x0
auipc a0, 0x100
ebreak

; check: Registers after execution:
; check: a0 = 1048576

# Test AUIPC: maximum positive immediate (0x7FFFF) at PC=0
# auipc a0, 0x7FFFF should load PC + (0x7FFFF << 12) = 0 + 0x7FFFF000 = 2147479552
.org 0x0
auipc a0, 0x7FFFF
ebreak

; check: Registers after execution:
; check: a0 = 2147479552

# Test AUIPC: positive immediate at non-zero PC
# auipc a0, 0x1 at PC=0x1000 should load 0x1000 + 4096 = 0x2000 = 8192
.org 0x1000
auipc a0, 0x1
ebreak

; check: Registers after execution:
; check: a0 = 8192

# Test AUIPC: larger positive immediate at non-zero PC
# auipc a0, 0x100 at PC=0x2000 should load 0x2000 + 0x100000 = 0x102000 = 1056768
.org 0x2000
auipc a0, 0x100
ebreak

; check: Registers after execution:
; check: a0 = 1056768

# Test AUIPC: positive immediate at high PC
# auipc a0, 0x1 at PC=0x10000 should load 0x10000 + 4096 = 0x11000 = 69632
.org 0x10000
auipc a0, 0x1
ebreak

; check: Registers after execution:
; check: a0 = 69632

# Test AUIPC: negative immediate (sign bit set) at PC=0
# auipc a0, 0x80000: sign_extend(0x80000) = 0xFFF80000, shifted = 0x80000000
# Result: PC (0) + 0x80000000 = 0x80000000 = -2147483648
.org 0x0
auipc a0, 0x80000
ebreak

; check: Registers after execution:
; check: a0 = -2147483648

# Test AUIPC: negative immediate at non-zero PC
# auipc a0, 0x80000 at PC=0x1000: 0x1000 + 0x80000000 = 0x80001000
# As signed i32: 0x80001000 as u32 = 2147487744, as i32 = -2147477504
# But wait, 0x80001000 = 2147487744, and 2147487744 - 0x100000000 = -2147477504
# Actually: 0x80001000 as i32 = -2147477504 ✓
.org 0x1000
auipc a0, 0x80000
ebreak

; check: Registers after execution:
; check: a0 = -2147479552

# Test AUIPC: maximum negative immediate (0xFFFFF) at PC=0
# auipc a0, 0xFFFFF: sign_extend(0xFFFFF) = 0xFFFFFFFF, shifted = 0xFFFFF000 = -4096
# Result: PC (0) + (-4096) = -4096
.org 0x0
auipc a0, 0xFFFFF
ebreak

; check: Registers after execution:
; check: a0 = -4096

# Test AUIPC: negative immediate (0xFF000) at PC=0
# auipc a0, 0xFF000: sign_extend(0xFF000) = 0xFFFFF000, shifted = 0xFFF000000
# Actually: 0xFF000 has bit 19 set, so sign-extends to 0xFFFFF000, shifted = 0xFFF000000
# In 32-bit: 0xFFF000000 wraps to 0xFF000000 = -16777216
.org 0x0
auipc a0, 0xFF000
ebreak

; check: Registers after execution:
; check: a0 = -16777216

# Test AUIPC: negative immediate at high PC
# auipc a0, 0xFFFFF at PC=0x10000: 0x10000 + (-4096) = 0x10000 - 4096 = 0xF000 = 61440
.org 0x10000
auipc a0, 0xFFFFF
ebreak

; check: Registers after execution:
; check: a0 = 61440

# Test AUIPC: small negative offset at PC
# auipc a0, 0x80000 at PC=0x8000: 0x8000 + 0x80000000 = 0x80008000
# As signed i32: 0x80008000 = 2147516416, as i32 = -2147459072
.org 0x8000
auipc a0, 0x80000
ebreak

; check: Registers after execution:
; check: a0 = -2147450880

# Test AUIPC: chained with ADDI (common pattern for loading addresses)
# auipc a0, 0x1 at PC=0x1000: a0 = 0x1000 + 4096 = 0x2000
# addi a0, a0, 0x234: a0 = 0x2000 + 564 = 0x2234 = 8756
.org 0x1000
auipc a0, 0x1
addi a0, a0, 0x234
ebreak

; check: Registers after execution:
; check: a0 = 8756

# Test AUIPC: multiple AUIPC instructions
# First executes at PC=0x2000, second at PC=0x2004, third at PC=0x2008
.org 0x2000
auipc a0, 0x1
auipc a1, 0x2
auipc a2, 0x3
ebreak

; check: Registers after execution:
; check: a0 = 12288
; check: a1 = 16388
; check: a2 = 20488

# Test AUIPC: positive immediate at PC=0x4000
.org 0x4000
auipc a0, 0x1
ebreak

; check: Registers after execution:
; check: a0 = 20480

# Test AUIPC: negative immediate at PC=0x5000
.org 0x5000
auipc a1, 0x80000
ebreak

; check: Registers after execution:
; check: a1 = -2147463168
