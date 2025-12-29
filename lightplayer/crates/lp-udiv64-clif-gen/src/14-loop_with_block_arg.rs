#![no_std]

/// Test 14: Loop that passes i64 as block argument
/// Tests if the bug is in how i64 values are passed between blocks
#[no_mangle]
pub fn divide64(value: u64) -> u64 {
    let mut v = value;
    
    // Simple loop that passes i64 as block argument
    let mut count = 0;
    loop {
        if count >= 2 {
            break;
        }
        count += 1;
        
        // Pass i64 to another "block" via function call pattern
        v = helper(v);
    }
    
    v
}

#[inline(never)]
fn helper(x: u64) -> u64 {
    // Just return it - tests if i64 block args work
    x
}

// run: divide64(0x0000000000000064) == 0x0000000000000064
// run: divide64(0x0000000000000000) == 0x0000000000000000

