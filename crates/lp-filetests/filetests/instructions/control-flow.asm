test instruction

# Test BEQ: branch if equal
addi a0, zero, 5
addi a1, zero, 5
beq a0, a1, target1
addi a2, zero, 0
ebreak
target1:
addi a2, zero, 1
ebreak

; check: Registers after execution:
; check: a0 = 5
; check: a1 = 5
; check: a2 = 1

# Test BNE: branch if not equal
addi a0, zero, 5
addi a1, zero, 3
bne a0, a1, target2
addi a2, zero, 0
ebreak
target2:
addi a2, zero, 1
ebreak

; check: Registers after execution:
; check: a0 = 5
; check: a1 = 3
; check: a2 = 1

# Test BLT: branch if less than
addi a0, zero, 3
addi a1, zero, 5
blt a0, a1, target3
addi a2, zero, 0
ebreak
target3:
addi a2, zero, 1
ebreak

; check: Registers after execution:
; check: a0 = 3
; check: a1 = 5
; check: a2 = 1

# Test BGE: branch if greater or equal
addi a0, zero, 5
addi a1, zero, 3
bge a0, a1, target4
addi a2, zero, 0
ebreak
target4:
addi a2, zero, 1
ebreak

; check: Registers after execution:
; check: a0 = 5
; check: a1 = 3
; check: a2 = 1

# Test JAL: jump and link
jal ra, target5
addi a0, zero, 0
ebreak
target5:
addi a0, zero, 1
ebreak

; check: Registers after execution:
; check: a0 = 1

# Test JALR: jump and link register (simplified - just test the instruction)
addi a0, zero, 0
addi a1, zero, 16  # Jump to offset 16 (4 instructions ahead)
jalr ra, 0(a1)
addi a0, zero, 0
addi a0, zero, 0
addi a0, zero, 0
addi a0, zero, 1
ebreak

; check: Registers after execution:
; check: a0 = 1

