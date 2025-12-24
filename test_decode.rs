use lp_riscv_tools::decode::decode_instruction;

fn main() {
    // Test SRLI with shift=32 (should be encoded as 0)
    // SRLI rd, rs1, 32 -> bits[11:5]=0, bits[4:0]=0, so immediate=0
    let srli_inst = 0x02005013; // srli a0, a0, 0 (but should be interpreted as shift by 32)
    println!("SRLI instruction: 0x{:08x}", srli_inst);
    match decode_instruction(srli_inst) {
        Ok(inst) => println!("Decoded: {:?}", inst),
        Err(e) => println!("Error: {}", e),
    }

    // Test SRAI with shift=24
    // SRAI rd, rs1, 24 -> bits[11:5]=0x20, bits[4:0]=24, so immediate=0x418 (1048 decimal)
    let srai_inst = 0x41805013; // srai a0, a0, 24
    println!("SRAI instruction: 0x{:08x}", srai_inst);
    match decode_instruction(srai_inst) {
        Ok(inst) => println!("Decoded: {:?}", inst),
        Err(e) => println!("Error: {}", e),
    }
}
