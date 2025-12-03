test instruction

# Test SLL: 5 << 2 = 20
addi a0, zero, 5
addi a1, zero, 2
sll a2, a0, a1
ebreak

; check: Registers after execution:
; check: a0 = 5
; check: a1 = 2
; check: a2 = 20

# Test SRL: 20 >> 2 = 5
addi a0, zero, 20
addi a1, zero, 2
srl a2, a0, a1
ebreak

; check: Registers after execution:
; check: a0 = 20
; check: a1 = 2
; check: a2 = 5

# Test SRA: -20 >> 2 = -5 (arithmetic right shift)
addi a0, zero, -20
addi a1, zero, 2
sra a2, a0, a1
ebreak

; check: Registers after execution:
; check: a0 = -20
; check: a1 = 2
; check: a2 = -5


