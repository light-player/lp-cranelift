// Quick test to verify object::Architecture::Riscv32 exists
fn main() {
    use object::Architecture;

    // Try to use Riscv32 to see if it exists
    let _arch = Architecture::Riscv32;

    println!("Riscv32 architecture exists in object crate!");
}

