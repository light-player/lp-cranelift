test instruction

# Test SLLI: 5 << 2 = 20
addi a0, zero, 5
slli a1, a0, 2
ebreak

; check: Registers after execution:
; check: a0 = 5
; check: a1 = 20

# Test SRLI: 20 >> 2 = 5
addi a0, zero, 20
srli a1, a0, 2
ebreak

; check: Registers after execution:
; check: a0 = 20
; check: a1 = 5

# Test SRAI: -20 >> 2 = -5 (arithmetic right shift)
addi a0, zero, -20
srai a1, a0, 2
ebreak

; check: Registers after execution:
; check: a0 = -20
; check: a1 = -5


