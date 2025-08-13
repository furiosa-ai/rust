//! Random number generation for peOS

pub fn fill_bytes(bytes: &mut [u8]) {
    // For peOS, use a simple PRNG
    // In a real implementation, this would use peOS entropy source
    static mut SEED: u64 = 1;
    
    unsafe {
        for byte in bytes.iter_mut() {
            // Simple linear congruential generator
            SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
            *byte = (SEED >> 32) as u8;
        }
    }
}