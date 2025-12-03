test instruction

# Test ANDI: 10 & 12 = 8
addi a0, zero, 10
andi a1, a0, 12
ebreak

; check: Registers after execution:
; check: a0 = 10
; check: a1 = 8

# Test ORI: 10 | 12 = 14
addi a0, zero, 10
ori a1, a0, 12
ebreak

; check: Registers after execution:
; check: a0 = 10
; check: a1 = 14

# Test XORI: 10 ^ 12 = 6
addi a0, zero, 10
xori a1, a0, 12
ebreak

; check: Registers after execution:
; check: a0 = 10
; check: a1 = 6


