test instruction

# Test SLT: 5 < 10 = 1 (true)
addi a0, zero, 5
addi a1, zero, 10
slt a2, a0, a1
ebreak

; check: Registers after execution:
; check: a0 = 5
; check: a1 = 10
; check: a2 = 1

# Test SLT: 10 < 5 = 0 (false)
addi a0, zero, 10
addi a1, zero, 5
slt a2, a0, a1
ebreak

; check: Registers after execution:
; check: a0 = 10
; check: a1 = 5
; check: a2 = 0

# Test SLTU: 5 < 10 = 1 (unsigned, true)
addi a0, zero, 5
addi a1, zero, 10
sltu a2, a0, a1
ebreak

; check: Registers after execution:
; check: a0 = 5
; check: a1 = 10
; check: a2 = 1


